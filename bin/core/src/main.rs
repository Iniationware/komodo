#![recursion_limit = "256"]

#[macro_use]
extern crate tracing;

use std::{net::SocketAddr, str::FromStr};

use anyhow::Context;
use axum::{Router, routing::get};
use axum_server::{Handle, tls_rustls::RustlsConfig};
use tower_http::{
  cors::{Any, CorsLayer, AllowOrigin},
  services::{ServeDir, ServeFile},
  set_header::SetResponseHeaderLayer,
};
use axum::http::{HeaderValue, header};
use tracing::Instrument;

use crate::config::{core_config, core_keys};

mod alert;
mod api;
mod auth;
mod cloud;
mod config;
mod connection;
mod helpers;
mod listener;
mod monitor;
mod network;
mod periphery;
mod permission;
mod resource;
mod schedule;
mod stack;
mod startup;
mod state;
mod sync;
mod ts_client;
mod ws;

/// Applies security headers to the router.
///
/// Adds security headers including:
/// - X-Content-Type-Options: nosniff
/// - X-Frame-Options: DENY
/// - X-XSS-Protection: 1; mode=block
/// - Referrer-Policy: strict-origin-when-cross-origin
/// - HSTS (if SSL enabled)
///
/// # Arguments
///
/// * `router` - The router to apply headers to
/// * `config` - The core configuration
///
/// # Returns
///
/// Router with security headers applied
fn apply_security_headers(
  router: Router,
  config: &komodo_client::entities::config::core::CoreConfig,
) -> Router {
  let mut router = router
    .layer(SetResponseHeaderLayer::overriding(
      header::X_CONTENT_TYPE_OPTIONS,
      HeaderValue::from_static("nosniff"),
    ))
    .layer(SetResponseHeaderLayer::overriding(
      header::HeaderName::from_static("x-frame-options"),
      HeaderValue::from_static("DENY"),
    ))
    .layer(SetResponseHeaderLayer::overriding(
      header::HeaderName::from_static("x-xss-protection"),
      HeaderValue::from_static("1; mode=block"),
    ))
    .layer(SetResponseHeaderLayer::overriding(
      header::HeaderName::from_static("referrer-policy"),
      HeaderValue::from_static("strict-origin-when-cross-origin"),
    ));

  // Add HSTS header if SSL is enabled
  if config.ssl_enabled {
    router = router.layer(SetResponseHeaderLayer::overriding(
      header::HeaderName::from_static("strict-transport-security"),
      HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    ));
  }

  router
}

/// Creates a CORS layer based on the configuration.
///
/// # Behavior
///
/// - If `cors_allowed_origins` is empty: Allows all origins (backward compatibility)
/// - If `cors_allowed_origins` is set: Only allows the specified origins
/// - Methods and headers are always allowed (Any)
/// - Credentials are only allowed if `cors_allow_credentials` is true
///
/// # Arguments
///
/// * `config` - The core configuration containing CORS settings
///
/// # Returns
///
/// A configured `CorsLayer` ready to be added to the Axum router
fn create_cors_layer(config: &komodo_client::entities::config::core::CoreConfig) -> CorsLayer {
  let mut cors = CorsLayer::new()
    .allow_methods(Any)
    .allow_headers(Any);

  if config.cors_allowed_origins.is_empty() {
    // If no origins specified, allow all (backward compatibility)
    cors = cors.allow_origin(Any);
  } else {
    // Allow specific origins
    let origins: Vec<HeaderValue> = config
      .cors_allowed_origins
      .iter()
      .filter_map(|origin| HeaderValue::from_str(origin).ok())
      .collect();
    cors = cors.allow_origin(AllowOrigin::list(origins));
  }

  if config.cors_allow_credentials {
    cors = cors.allow_credentials(true);
  }

  cors
}

async fn app() -> anyhow::Result<()> {
  dotenvy::dotenv().ok();
  let config = core_config();
  logger::init(&config.logging)?;

  let startup_span = info_span!("CoreStartup");

  async {
    info!("Komodo Core version: v{}", env!("CARGO_PKG_VERSION"));

    match (
      config.pretty_startup_config,
      config.unsafe_unsanitized_startup_config,
    ) {
      (true, true) => info!("{:#?}", config),
      (true, false) => info!("{:#?}", config.sanitized()),
      (false, true) => info!("{:?}", config),
      (false, false) => info!("{:?}", config.sanitized()),
    }

    // Init + log public key. Will crash if invalid private key here.
    info!("Public Key: {}", core_keys().load().public);

    rustls::crypto::aws_lc_rs::default_provider()
      .install_default()
      .context("Failed to install default crypto provider")?;

    // Init jwt client to crash on failure
    state::jwt_client();
    tokio::join!(
      // Init db_client check to crash on db init failure
      async {
        if let Err(e) = state::init_db_client().await {
          error!("Failed to initialize database client: {e:#}");
          panic!("Database initialization failed: {e:#}");
        }
      },
      // Manage OIDC client (defined in config / env vars / compose secret file)
      auth::oidc::client::spawn_oidc_client_management()
    );
    // Run after db connection.
    startup::on_startup().await;

    // Spawn background tasks
    monitor::spawn_monitor_loop();
    resource::spawn_resource_refresh_loop();
    resource::spawn_all_resources_cache_refresh_loop();
    resource::spawn_build_state_refresh_loop();
    resource::spawn_repo_state_refresh_loop();
    resource::spawn_procedure_state_refresh_loop();
    resource::spawn_action_state_refresh_loop();
    schedule::spawn_schedule_executor();
    helpers::prune::spawn_prune_loop();
  }
  .instrument(startup_span)
  .await;

  // Setup static frontend services
  let frontend_path = &config.frontend_path;
  let frontend_index =
    ServeFile::new(format!("{frontend_path}/index.html"));
  let serve_frontend = ServeDir::new(frontend_path)
    .not_found_service(frontend_index.clone());

  let app = Router::new()
    .route("/version", get(|| async { env!("CARGO_PKG_VERSION") }))
    .nest("/auth", api::auth::router())
    .nest("/user", api::user::router())
    .nest("/read", api::read::router())
    .nest("/write", api::write::router())
    .nest("/execute", api::execute::router())
    .nest("/terminal", api::terminal::router())
    .nest("/listener", listener::router())
    .nest("/ws", ws::router())
    .nest("/client", ts_client::router())
    .fallback_service(serve_frontend)
    .layer(create_cors_layer(&config));

  let app = apply_security_headers(app, &config);
  let app = app.into_make_service();

  let addr =
    format!("{}:{}", core_config().bind_ip, core_config().port);
  let socket_addr = SocketAddr::from_str(&addr)
    .context("failed to parse listen address")?;

  let handle = Handle::new();
  tokio::spawn({
    // Cannot run actions until the server is available.
    // We can use a handle for the server, and wait until
    // the handle is listening before running actions
    let handle = handle.clone();
    async move {
      handle.listening().await;
      startup::run_startup_actions().await;
    }
  });

  if config.ssl_enabled {
    info!("🔒 Core SSL Enabled");
    info!("Komodo Core starting on https://{socket_addr}");
    let ssl_config = RustlsConfig::from_pem_file(
      &config.ssl_cert_file,
      &config.ssl_key_file,
    )
    .await
    .context("Invalid ssl cert / key")?;
    axum_server::bind_rustls(socket_addr, ssl_config)
      .handle(handle)
      .serve(app)
      .await
      .context("failed to start https server")
  } else {
    info!("🔓 Core SSL Disabled");
    info!("Komodo Core starting on http://{socket_addr}");
    axum_server::bind(socket_addr)
      .handle(handle)
      .serve(app)
      .await
      .context("failed to start http server")
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let mut term_signal = tokio::signal::unix::signal(
    tokio::signal::unix::SignalKind::terminate(),
  )?;
  tokio::select! {
    res = tokio::spawn(app()) => res?,
    _ = term_signal.recv() => Ok(()),
  }
}
