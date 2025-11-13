//! # Input Validation Module
//!
//! This module provides validation functions for user inputs to prevent
//! invalid data from entering the system and improve security.
//!
//! ## Usage
//!
//! ```rust
//! use crate::helpers::validation::validate_username;
//!
//! validate_username("myuser")?;
//! ```

use anyhow::{anyhow, Result};
use regex::Regex;
use std::sync::OnceLock;

/// Maximum length for usernames
pub const MAX_USERNAME_LENGTH: usize = 100;
/// Minimum length for usernames
pub const MIN_USERNAME_LENGTH: usize = 1;
/// Maximum length for passwords
pub const MAX_PASSWORD_LENGTH: usize = 1000;
/// Minimum length for passwords
pub const MIN_PASSWORD_LENGTH: usize = 8;
/// Maximum length for API key names
pub const MAX_API_KEY_NAME_LENGTH: usize = 200;
/// Maximum length for variable names
pub const MAX_VARIABLE_NAME_LENGTH: usize = 500;
/// Maximum length for variable values
pub const MAX_VARIABLE_VALUE_LENGTH: usize = 10000;
/// Maximum length for URLs
pub const MAX_URL_LENGTH: usize = 2048;

/// Validates a username format and length.
///
/// # Arguments
///
/// * `username` - The username string to validate
///
/// # Returns
///
/// Returns `Ok(())` if the username is valid, or an error with details if invalid.
///
/// # Validation Rules
///
/// - Must be between 1 and 100 characters
/// - Can only contain alphanumeric characters, underscores, hyphens, dots, and @
/// - Useful for both simple usernames and email addresses
///
/// # Example
///
/// ```rust
/// use crate::helpers::validation::validate_username;
///
/// assert!(validate_username("user123").is_ok());
/// assert!(validate_username("user@example.com").is_ok());
/// assert!(validate_username("").is_err());
/// assert!(validate_username("user name").is_err()); // Contains space
/// ```
pub fn validate_username(username: &str) -> Result<()> {
  if username.is_empty() {
    return Err(anyhow!("Username cannot be empty"));
  }

  if username.len() < MIN_USERNAME_LENGTH {
    return Err(anyhow!(
      "Username must be at least {} characters long",
      MIN_USERNAME_LENGTH
    ));
  }

  if username.len() > MAX_USERNAME_LENGTH {
    return Err(anyhow!(
      "Username must be at most {} characters long",
      MAX_USERNAME_LENGTH
    ));
  }

  // Username should only contain alphanumeric characters, underscores, hyphens, dots, and @
  // This allows email addresses and simple usernames
  static USERNAME_REGEX: OnceLock<Regex> = OnceLock::new();
  let regex = USERNAME_REGEX.get_or_init(|| {
    Regex::new(r"^[a-zA-Z0-9._@-]+$")
      .expect("Failed to compile username regex")
  });

  if !regex.is_match(username) {
    return Err(anyhow!(
      "Username contains invalid characters. Only alphanumeric characters, underscores, hyphens, dots, and @ are allowed"
    ));
  }

  Ok(())
}

/// Validates a password format and length.
///
/// # Arguments
///
/// * `password` - The password string to validate
///
/// # Returns
///
/// Returns `Ok(())` if the password is valid, or an error with details if invalid.
///
/// # Validation Rules
///
/// - Must be between 8 and 1000 characters
/// - Cannot be empty
///
/// # Example
///
/// ```rust
/// use crate::helpers::validation::validate_password;
///
/// assert!(validate_password("securepass123").is_ok());
/// assert!(validate_password("short").is_err()); // Too short
/// assert!(validate_password("").is_err()); // Empty
/// ```
pub fn validate_password(password: &str) -> Result<()> {
  if password.is_empty() {
    return Err(anyhow!("Password cannot be empty"));
  }

  if password.len() < MIN_PASSWORD_LENGTH {
    return Err(anyhow!(
      "Password must be at least {} characters long",
      MIN_PASSWORD_LENGTH
    ));
  }

  if password.len() > MAX_PASSWORD_LENGTH {
    return Err(anyhow!(
      "Password must be at most {} characters long",
      MAX_PASSWORD_LENGTH
    ));
  }

  Ok(())
}

/// Validates a URL format and length.
///
/// # Arguments
///
/// * `url` - The URL string to validate
///
/// # Returns
///
/// Returns `Ok(())` if the URL is valid, or an error with details if invalid.
///
/// # Validation Rules
///
/// - Must start with `http://` or `https://`
/// - Must be a valid URL format (parsed by `url::Url`)
/// - Maximum length: 2048 characters
///
/// # Example
///
/// ```rust
/// use crate::helpers::validation::validate_url;
///
/// assert!(validate_url("https://example.com").is_ok());
/// assert!(validate_url("http://localhost:8080").is_ok());
/// assert!(validate_url("invalid-url").is_err()); // Missing protocol
/// ```
pub fn validate_url(url: &str) -> Result<()> {
  if url.is_empty() {
    return Err(anyhow!("URL cannot be empty"));
  }

  if url.len() > MAX_URL_LENGTH {
    return Err(anyhow!(
      "URL must be at most {} characters long",
      MAX_URL_LENGTH
    ));
  }

  // Basic URL validation - check if it starts with http:// or https://
  if !url.starts_with("http://") && !url.starts_with("https://") {
    return Err(anyhow!("URL must start with http:// or https://"));
  }

  // Try to parse as URL
  url::Url::parse(url)
    .map_err(|e| anyhow!("Invalid URL format: {e}"))?;

  Ok(())
}

/// Validates an API key name.
///
/// # Arguments
///
/// * `name` - The API key name string to validate
///
/// # Returns
///
/// Returns `Ok(())` if the name is valid, or an error with details if invalid.
///
/// # Validation Rules
///
/// - Cannot be empty
/// - Maximum length: 200 characters
///
/// # Example
///
/// ```rust
/// use crate::helpers::validation::validate_api_key_name;
///
/// assert!(validate_api_key_name("My API Key").is_ok());
/// assert!(validate_api_key_name("").is_err()); // Empty
/// ```
pub fn validate_api_key_name(name: &str) -> Result<()> {
  if name.is_empty() {
    return Err(anyhow!("API key name cannot be empty"));
  }

  if name.len() > MAX_API_KEY_NAME_LENGTH {
    return Err(anyhow!(
      "API key name must be at most {} characters long",
      MAX_API_KEY_NAME_LENGTH
    ));
  }

  Ok(())
}

/// Validates a variable name.
///
/// # Arguments
///
/// * `name` - The variable name string to validate
///
/// # Returns
///
/// Returns `Ok(())` if the variable name is valid, or an error with details if invalid.
///
/// # Validation Rules
///
/// - Must start with a letter or underscore
/// - Can only contain alphanumeric characters and underscores
/// - Maximum length: 500 characters
/// - Cannot be empty
///
/// # Example
///
/// ```rust
/// use crate::helpers::validation::validate_variable_name;
///
/// assert!(validate_variable_name("MY_VAR").is_ok());
/// assert!(validate_variable_name("_private").is_ok());
/// assert!(validate_variable_name("123var").is_err()); // Starts with number
/// assert!(validate_variable_name("var-name").is_err()); // Contains hyphen
/// ```
pub fn validate_variable_name(name: &str) -> Result<()> {
  if name.is_empty() {
    return Err(anyhow!("Variable name cannot be empty"));
  }

  if name.len() > MAX_VARIABLE_NAME_LENGTH {
    return Err(anyhow!(
      "Variable name must be at most {} characters long",
      MAX_VARIABLE_NAME_LENGTH
    ));
  }

  // Variable names should be valid identifiers
  static VAR_NAME_REGEX: OnceLock<Regex> = OnceLock::new();
  let regex = VAR_NAME_REGEX.get_or_init(|| {
    Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$")
      .expect("Failed to compile variable name regex")
  });

  if !regex.is_match(name) {
    return Err(anyhow!(
      "Variable name must start with a letter or underscore and contain only alphanumeric characters and underscores"
    ));
  }

  Ok(())
}

/// Sanitizes input to prevent NoSQL injection attacks.
///
/// While MongoDB's BSON serialization provides some protection,
/// this function adds an extra layer by removing or escaping
/// potentially dangerous characters.
///
/// # Arguments
///
/// * `input` - The input string to sanitize
///
/// # Returns
///
/// Sanitized string safe for use in MongoDB queries
///
/// # Security Note
///
/// This is a defense-in-depth measure. The primary protection
/// comes from using parameterized queries with `doc!` macros,
/// which serialize values as BSON literals rather than code.
pub fn sanitize_for_mongodb(input: &str) -> String {
  // Remove null bytes and control characters
  input
    .chars()
    .filter(|c| !c.is_control() && *c != '\0')
    .collect()
}

/// Validates that a string is safe for use as a MongoDB field value.
///
/// Checks for:
/// - Null bytes
/// - Control characters
/// - Excessive length (prevent DoS)
///
/// # Arguments
///
/// * `value` - The value to validate
/// * `max_length` - Maximum allowed length
///
/// # Returns
///
/// `Ok(())` if safe, `Err` with details if unsafe
pub fn validate_mongodb_field_value(
  value: &str,
  max_length: usize,
) -> Result<()> {
  if value.is_empty() {
    return Err(anyhow!("Value cannot be empty"));
  }

  if value.len() > max_length {
    return Err(anyhow!(
      "Value exceeds maximum length of {} characters",
      max_length
    ));
  }

  // Check for null bytes and control characters
  if value.chars().any(|c| c.is_control() || c == '\0') {
    return Err(anyhow!("Value contains invalid control characters"));
  }

  Ok(())
}

/// Validates a variable value
pub fn validate_variable_value(value: &str) -> Result<()> {
  if value.len() > MAX_VARIABLE_VALUE_LENGTH {
    return Err(anyhow!(
      "Variable value must be at most {} characters long",
      MAX_VARIABLE_VALUE_LENGTH
    ));
  }

  Ok(())
}

/// Validates a string length is within bounds
pub fn validate_string_length(
  value: &str,
  field_name: &str,
  min: Option<usize>,
  max: Option<usize>,
) -> Result<()> {
  if let Some(min_len) = min {
    if value.len() < min_len {
      return Err(anyhow!(
        "{field_name} must be at least {min_len} characters long"
      ));
    }
  }

  if let Some(max_len) = max {
    if value.len() > max_len {
      return Err(anyhow!(
        "{field_name} must be at most {max_len} characters long"
      ));
    }
  }

  Ok(())
}

