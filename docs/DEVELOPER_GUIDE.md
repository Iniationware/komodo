# Komodo Developer Guide

**Version:** 1.0  
**Last Updated:** $(date)

---

## 📚 Table of Contents

1. [Security Features](#security-features)
2. [Input Validation](#input-validation)
3. [Rate Limiting](#rate-limiting)
4. [Security Headers](#security-headers)
5. [Error Handling](#error-handling)
6. [API Reference](#api-reference)

---

## 🔒 Security Features

### Overview

Komodo implements comprehensive security features to protect against common web vulnerabilities and attacks.

### CORS Configuration

**Location:** `bin/core/src/main.rs`

CORS (Cross-Origin Resource Sharing) can be configured via environment variables or config file.

**Configuration:**

```toml
# config/core.config.toml
cors_allowed_origins = ["https://komodo.example.com"]
cors_allow_credentials = false
```

**Environment Variables:**
```bash
KOMODO_CORS_ALLOWED_ORIGINS=https://komodo.example.com,https://app.example.com
KOMODO_CORS_ALLOW_CREDENTIALS=false
```

**Behavior:**
- If `cors_allowed_origins` is empty: Allows all origins (backward compatibility)
- If `cors_allowed_origins` is set: Only allows specified origins
- Methods and headers: Always allowed (Any)
- Credentials: Only allowed if `cors_allow_credentials` is true

**Code Example:**
```rust
use crate::config::core_config;

let config = core_config();
// CORS layer is automatically applied in main.rs
```

---

### Security Headers

**Location:** `bin/core/src/main.rs`

Security headers are automatically added to all HTTP responses.

**Implemented Headers:**

| Header | Value | Purpose |
|--------|-------|---------|
| `X-Content-Type-Options` | `nosniff` | Prevents MIME-type sniffing |
| `X-Frame-Options` | `DENY` | Prevents clickjacking attacks |
| `X-XSS-Protection` | `1; mode=block` | XSS protection (legacy browsers) |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Controls referrer information |
| `Strict-Transport-Security` | `max-age=31536000; includeSubDomains` | HSTS (only when SSL enabled) |

**Implementation:**
```rust
fn apply_security_headers(router: Router, config: &CoreConfig) -> Router {
  // Headers are automatically applied to all responses
}
```

---

## ✅ Input Validation

### Overview

**Location:** `bin/core/src/helpers/validation.rs`

The validation module provides comprehensive input validation functions to prevent invalid data and improve security.

### Available Functions

#### `validate_username(username: &str) -> Result<()>`

Validates username format and length.

**Rules:**
- Length: 1-100 characters
- Allowed characters: alphanumeric, underscores, hyphens, dots, @
- Useful for both simple usernames and email addresses

**Example:**
```rust
use crate::helpers::validation::validate_username;

validate_username("user123")?;
validate_username("user@example.com")?;
```

#### `validate_password(password: &str) -> Result<()>`

Validates password format and length.

**Rules:**
- Minimum length: 8 characters
- Maximum length: 1000 characters
- Cannot be empty

**Example:**
```rust
use crate::helpers::validation::validate_password;

validate_password("securepass123")?;
```

#### `validate_url(url: &str) -> Result<()>`

Validates URL format and length.

**Rules:**
- Must start with `http://` or `https://`
- Must be valid URL format (parsed by `url::Url`)
- Maximum length: 2048 characters

**Example:**
```rust
use crate::helpers::validation::validate_url;

validate_url("https://example.com")?;
```

#### `validate_api_key_name(name: &str) -> Result<()>`

Validates API key name.

**Rules:**
- Cannot be empty
- Maximum length: 200 characters

#### `validate_variable_name(name: &str) -> Result<()>`

Validates variable name format.

**Rules:**
- Must start with letter or underscore
- Can only contain alphanumeric characters and underscores
- Maximum length: 500 characters

#### `sanitize_for_mongodb(input: &str) -> String`

Sanitizes input to prevent NoSQL injection attacks.

**Behavior:**
- Removes null bytes (`\0`)
- Removes control characters
- Returns sanitized string

**Security Note:**
This is a defense-in-depth measure. The primary protection comes from using parameterized queries with `doc!` macros.

**Example:**
```rust
use crate::helpers::validation::sanitize_for_mongodb;

let sanitized = sanitize_for_mongodb(user_input);
```

#### `validate_mongodb_field_value(value: &str, max_length: usize) -> Result<()>`

Validates that a string is safe for use as a MongoDB field value.

**Checks:**
- Not empty
- Length within limits
- No null bytes or control characters

---

## 🚦 Rate Limiting

### Overview

**Location:** `bin/core/src/helpers/security.rs`

Rate limiting protects authentication endpoints from brute-force attacks.

### Configuration

**Default Settings:**
- Max attempts: 5 per IP
- Time window: 5 minutes (300 seconds)
- Applies to: All `/auth/*` endpoints

### Usage

Rate limiting is automatically applied to authentication endpoints:

```rust
// bin/core/src/api/auth.rs
pub fn router() -> Router {
  let rate_limiter = RateLimiter::new(5, 300);
  // Middleware automatically applied
}
```

### Custom Rate Limiter

```rust
use crate::helpers::security::RateLimiter;

let limiter = RateLimiter::new(
  10,  // max attempts
  600  // window in seconds (10 minutes)
);

limiter.check(&ip_address)?;
```

### Behavior

- **On rate limit exceeded:** Returns HTTP 429 (Too Many Requests)
- **On successful login:** Automatically resets rate limit for that IP
- **Logging:** All rate limit events are logged as security events

---

## 📝 Security Logging

### Overview

**Location:** `bin/core/src/helpers/security.rs`

Security events are automatically logged for audit and monitoring purposes.

### Logged Events

| Event Type | Description | Log Level |
|------------|-------------|-----------|
| `failed_login` | Failed login attempt | WARN |
| `failed_login_query` | Database error during login | WARN |
| `rate_limit_exceeded` | Rate limit exceeded | WARN |
| Successful login | Successful authentication | INFO |

### Usage

```rust
use crate::helpers::security::{get_client_ip, log_security_event};

let ip = get_client_ip(&headers);
log_security_event("failed_login", "Invalid password", &ip);
```

### Log Format

```
SECURITY EVENT | type: failed_login | ip: 192.168.1.1 | details: Invalid password
```

---

## 🔐 Session Management

### JWT Token Validation

**Location:** `bin/core/src/auth/mod.rs`

JWT tokens include clock skew tolerance to handle minor time differences between servers.

**Clock Skew Tolerance:**
- JWT tokens: 5 minutes
- Exchange tokens: 1 minute
- API keys: 5 minutes

**Implementation:**
```rust
const CLOCK_SKEW_TOLERANCE_MS: u128 = 5 * 60 * 1000;

// Token is valid if expiration > (now - tolerance)
if claims.exp > now.saturating_sub(CLOCK_SKEW_TOLERANCE_MS) {
  Ok(claims.id)
}
```

---

## ⚠️ Error Handling

### Best Practices

**Location:** Throughout codebase

Error handling has been improved to provide better diagnostics and prevent crashes.

**Patterns:**

1. **Replace `unwrap()` with proper error handling:**
```rust
// Before
let value = something.unwrap();

// After
let value = something.context("Failed to get value")?;
```

2. **Use `unwrap_or_else` for fallbacks:**
```rust
let config = load_config().unwrap_or_else(|e| {
  error!("Failed to load config: {e:#}");
  Default::default()
});
```

3. **Provide context with `anyhow::Context`:**
```rust
let result = operation()
  .context("Failed to perform operation")?;
```

---

## 📖 API Reference

### Security Helpers

#### `RateLimiter`

```rust
pub struct RateLimiter {
  // Internal state
}

impl RateLimiter {
  pub fn new(max_attempts: usize, window_seconds: u64) -> Self;
  pub fn check(&self, key: &str) -> anyhow::Result<()>;
  pub fn reset(&self, key: &str);
}
```

#### `get_client_ip(headers: &HeaderMap) -> String`

Extracts client IP from request headers.

**Priority:**
1. `X-Forwarded-For` (first IP in chain)
2. `X-Real-IP`
3. `"unknown"` (fallback)

#### `log_security_event(event_type: &str, details: &str, ip: &str)`

Logs a security event.

---

## 🛠️ Development Guidelines

### Adding New Validation

1. Add validation function to `bin/core/src/helpers/validation.rs`
2. Add constants for limits at top of file
3. Add comprehensive doc-comments
4. Include examples in doc-comments
5. Apply validation in relevant endpoints

### Adding Security Features

1. Add security logic to `bin/core/src/helpers/security.rs`
2. Document security implications
3. Add security logging where appropriate
4. Update this guide

### Error Handling

1. Never use `unwrap()` in production code paths
2. Always provide context with errors
3. Use `anyhow::Context` for error chaining
4. Log errors at appropriate levels

---

## 📚 Additional Resources

- [Security Improvements Documentation](../SECURITY_IMPROVEMENTS.md)
- [Race Condition Analysis](../RACE_CONDITION_ANALYSE.md)
- [Code Analysis Report](../ANALYSE_BERICHT.md)

---

## 🔄 Changelog

### Security Improvements (Latest)

- ✅ CORS configuration made configurable
- ✅ Security headers implemented
- ✅ Rate limiting for auth endpoints
- ✅ Input validation library created
- ✅ NoSQL injection protection added
- ✅ Security logging implemented
- ✅ JWT clock skew tolerance added

---

**For questions or contributions, please refer to the main project documentation.**

