use std::str::FromStr;

use anyhow::{Context, anyhow};
use async_timing_util::unix_timestamp_ms;
use database::{
  hash_password,
  mungos::mongodb::bson::{Document, doc, oid::ObjectId},
};
use komodo_client::{
  api::auth::{
    LoginLocalUser, LoginLocalUserResponse, SignUpLocalUser,
    SignUpLocalUserResponse,
  },
  entities::user::{User, UserConfig},
};
use resolver_api::Resolve;

use crate::{
  api::auth::AuthArgs,
  config::core_config,
  helpers::{
    security::{get_client_ip, log_security_event},
    validation::{
      sanitize_for_mongodb, validate_mongodb_field_value,
      validate_password, validate_username,
    },
  },
  state::{db_client, jwt_client},
};

impl Resolve<AuthArgs> for SignUpLocalUser {
  #[instrument("SignUpLocalUser", skip(self))]
  async fn resolve(
    self,
    _: &AuthArgs,
  ) -> serror::Result<SignUpLocalUserResponse> {
    let core_config = core_config();

    if !core_config.local_auth {
      return Err(anyhow!("Local auth is not enabled").into());
    }

    // Validate username format and length
    validate_username(&self.username)
      .context("Invalid username format")?;

    // Additional MongoDB safety check
    validate_mongodb_field_value(&self.username, 100)
      .context("Username contains invalid characters")?;

    // Sanitize username for MongoDB
    let username = sanitize_for_mongodb(&self.username);

    if ObjectId::from_str(&username).is_ok() {
      return Err(
        anyhow!("Username cannot be valid ObjectId").into(),
      );
    }

    // Validate password format and length
    validate_password(&self.password)
      .context("Invalid password format")?;

    let db = db_client();

    let no_users_exist =
      db.users.find_one(Document::new()).await?.is_none();

    if !no_users_exist && core_config.disable_user_registration {
      return Err(anyhow!("User registration is disabled").into());
    }

    if db
      .users
      .find_one(doc! { "username": &username })
      .await
      .context("Failed to query for existing users")?
      .is_some()
    {
      return Err(anyhow!("Username already taken.").into());
    }

    let ts = unix_timestamp_ms() as i64;
    let hashed_password = hash_password(self.password)?;

    let user = User {
      id: Default::default(),
      username: username.clone(),
      enabled: no_users_exist || core_config.enable_new_users,
      admin: no_users_exist,
      super_admin: no_users_exist,
      create_server_permissions: no_users_exist,
      create_build_permissions: no_users_exist,
      updated_at: ts,
      last_update_view: 0,
      recents: Default::default(),
      all: Default::default(),
      config: UserConfig::Local {
        password: hashed_password,
      },
    };

    let user_id = db_client()
      .users
      .insert_one(user)
      .await
      .context("failed to create user")?
      .inserted_id
      .as_object_id()
      .context("inserted_id is not ObjectId")?
      .to_string();

    jwt_client()
      .encode(user_id.clone())
      .context("failed to generate jwt for user")
      .map_err(Into::into)
  }
}

impl Resolve<AuthArgs> for LoginLocalUser {
  #[instrument("LoginLocalUser", skip(self))]
  async fn resolve(
    self,
    args: &AuthArgs,
  ) -> serror::Result<LoginLocalUserResponse> {
    if !core_config().local_auth {
      return Err(anyhow!("local auth is not enabled").into());
    }

    let ip = get_client_ip(&args.headers);

    // Sanitize username for MongoDB query
    let username = sanitize_for_mongodb(&self.username);
    validate_mongodb_field_value(&username, 100)
      .context("Username contains invalid characters")?;

    let user = db_client()
      .users
      .find_one(doc! { "username": &username })
      .await
      .context("failed at db query for users")
      .map_err(|e| {
        log_security_event(
          "failed_login_query",
          &format!("Database query failed: {e:#}"),
          &ip,
        );
        e
      })?;

    let user = match user {
      Some(user) => user,
      None => {
        log_security_event(
          "failed_login",
          &format!("User not found: {}", self.username),
          &ip,
        );
        return Err(
          anyhow!("invalid credentials")
            .context("did not find user with username")
            .into(),
        );
      }
    };

    let UserConfig::Local {
      password: user_pw_hash,
    } = user.config
    else {
      log_security_event(
        "failed_login",
        &format!("Non-local auth user attempted password login: {}", self.username),
        &ip,
      );
      return Err(
        anyhow!(
          "non-local auth users can not log in with a password"
        )
        .into(),
      );
    };

    let verified = bcrypt::verify(self.password, &user_pw_hash)
      .context("failed at verify password")?;

    if !verified {
      log_security_event(
        "failed_login",
        &format!("Invalid password for user: {}", self.username),
        &ip,
      );
      return Err(anyhow!("invalid credentials").into());
    }

    // Successful login - log security event
    info!("Successful login | user: {} | ip: {}", self.username, ip);

    jwt_client()
      .encode(user.id.clone())
      .context("failed at generating jwt for user")
      .map_err(Into::into)
  }
}
