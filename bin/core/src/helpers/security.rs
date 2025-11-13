//! # Security Helpers Module
//!
//! This module provides security-related utilities including:
//! - Rate limiting for authentication endpoints
//! - Security event logging
//! - Failed login attempt tracking

use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

use anyhow::anyhow;
use axum::{
  extract::Request,
  http::HeaderMap,
  middleware::Next,
  response::Response,
};
use reqwest::StatusCode;
use serror::AddStatusCode;

/// Rate limiter for authentication endpoints
#[derive(Clone)]
pub struct RateLimiter {
  attempts: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
  max_attempts: usize,
  window_seconds: u64,
}

impl RateLimiter {
  /// Create a new rate limiter
  ///
  /// # Arguments
  ///
  /// * `max_attempts` - Maximum number of attempts allowed
  /// * `window_seconds` - Time window in seconds
  pub fn new(max_attempts: usize, window_seconds: u64) -> Self {
    Self {
      attempts: Arc::new(Mutex::new(HashMap::new())),
      max_attempts,
      window_seconds,
    }
  }

  /// Check if an IP address is rate limited
  ///
  /// # Arguments
  ///
  /// * `key` - Identifier for rate limiting (e.g., IP address or username)
  ///
  /// # Returns
  ///
  /// `Ok(())` if not rate limited, `Err` if rate limited
  pub fn check(&self, key: &str) -> anyhow::Result<()> {
    let mut attempts = self.attempts.lock().map_err(|e| {
      anyhow!("Rate limiter lock poisoned: {e:?}")
    })?;

    let now = Instant::now();
    let window_start = now - Duration::from_secs(self.window_seconds);

    // Clean up old attempts
    let entry = attempts.entry(key.to_string()).or_insert_with(Vec::new);
    entry.retain(|&time| time > window_start);

    // Check if rate limited
    if entry.len() >= self.max_attempts {
      Err(anyhow!(
        "Too many attempts. Please try again in {} seconds.",
        self.window_seconds
      ))
    } else {
      entry.push(now);
      Ok(())
    }
  }

  /// Reset attempts for a key (e.g., after successful login)
  pub fn reset(&self, key: &str) {
    if let Ok(mut attempts) = self.attempts.lock() {
      attempts.remove(key);
    }
  }
}

/// Get client IP from request headers
///
/// Checks common headers for the real client IP:
/// - X-Forwarded-For
/// - X-Real-IP
/// - Remote address from connection info
pub fn get_client_ip(headers: &HeaderMap) -> String {
  // Check X-Forwarded-For header (first IP in chain)
  if let Some(forwarded) = headers.get("x-forwarded-for") {
    if let Ok(forwarded_str) = forwarded.to_str() {
      if let Some(ip) = forwarded_str.split(',').next() {
        return ip.trim().to_string();
      }
    }
  }

  // Check X-Real-IP header
  if let Some(real_ip) = headers.get("x-real-ip") {
    if let Ok(real_ip_str) = real_ip.to_str() {
      return real_ip_str.to_string();
    }
  }

  // Fallback to "unknown"
  "unknown".to_string()
}

/// Rate limiting middleware for authentication endpoints.
///
/// This middleware checks if the client IP has exceeded the rate limit
/// before processing the request. If rate limited, returns HTTP 429.
/// On successful authentication (HTTP 200), the rate limit is reset.
///
/// # Behavior
///
/// - Checks rate limit before processing request
/// - Returns HTTP 429 if rate limit exceeded
/// - Resets rate limit on successful authentication
/// - Logs rate limit events as security events
///
/// # Arguments
///
/// * `req` - The incoming HTTP request
/// * `next` - The next middleware/handler in the chain
///
/// # Returns
///
/// HTTP response, or error if rate limited
///
/// # Example
///
/// ```rust
/// use crate::helpers::security::rate_limit_middleware;
///
/// // Middleware is automatically applied to auth router
/// ```
pub async fn rate_limit_middleware(
  req: Request,
  next: Next,
) -> Result<Response, serror::Error> {
  // Get rate limiter from extensions or create default
  let rate_limiter = req
    .extensions()
    .get::<RateLimiter>()
    .cloned()
    .unwrap_or_else(|| RateLimiter::new(5, 300)); // 5 attempts per 5 minutes default

  let ip = get_client_ip(req.headers());

  rate_limiter
    .check(&ip)
    .map_err(|e| {
      log_security_event("rate_limit_exceeded", &format!("IP: {ip}"), &ip);
      e.status_code(StatusCode::TOO_MANY_REQUESTS)
    })?;

  let res = next.run(req).await;

  // Reset rate limit on successful authentication (status 200)
  if res.status() == StatusCode::OK {
    rate_limiter.reset(&ip);
  }

  Ok(res)
}

/// Log security event
///
/// # Arguments
///
/// * `event_type` - Type of security event (e.g., "failed_login", "rate_limit")
/// * `details` - Additional details about the event
/// * `ip` - Client IP address
pub fn log_security_event(event_type: &str, details: &str, ip: &str) {
  warn!(
    "SECURITY EVENT | type: {} | ip: {} | details: {}",
    event_type, ip, details
  );
}

