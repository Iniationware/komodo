# Security Changelog

This document tracks security-related changes and improvements to Komodo.

---

## [Unreleased] - Security Improvements

### Added

#### Security Features
- **CORS Configuration** - Configurable CORS origins via environment variables or config file
  - Environment variable: `KOMODO_CORS_ALLOWED_ORIGINS`
  - Config option: `cors_allowed_origins`
  - Backward compatible (empty list = allow all)

- **Security Headers** - Automatic security headers on all HTTP responses
  - `X-Content-Type-Options: nosniff`
  - `X-Frame-Options: DENY`
  - `X-XSS-Protection: 1; mode=block`
  - `Referrer-Policy: strict-origin-when-cross-origin`
  - `Strict-Transport-Security` (HSTS, only when SSL enabled)

- **Rate Limiting** - Protection against brute-force attacks
  - 5 attempts per 5 minutes per IP address
  - Applied to all `/auth/*` endpoints
  - Automatic reset on successful authentication
  - HTTP 429 response on rate limit exceeded

- **Input Validation Library** - Comprehensive input validation
  - Username validation (format, length)
  - Password validation (minimum length: 8)
  - URL validation
  - API key name validation
  - Variable name/value validation
  - MongoDB field value validation

- **NoSQL Injection Protection** - Defense-in-depth measures
  - `sanitize_for_mongodb()` function
  - `validate_mongodb_field_value()` function
  - Applied to critical user inputs (usernames)

- **Security Logging** - Comprehensive security event logging
  - Failed login attempts
  - Rate limit exceeded events
  - Database query errors
  - IP-based tracking
  - Structured log format

- **Session Management Improvements** - Clock skew tolerance
  - JWT tokens: 5 minute clock skew tolerance
  - Exchange tokens: 1 minute clock skew tolerance
  - API keys: 5 minute clock skew tolerance

#### New Modules
- `bin/core/src/helpers/security.rs` - Security helper functions
  - `RateLimiter` struct
  - `get_client_ip()` function
  - `log_security_event()` function
  - `rate_limit_middleware()` middleware

- `bin/core/src/helpers/validation.rs` - Input validation functions
  - `validate_username()`
  - `validate_password()`
  - `validate_url()`
  - `validate_api_key_name()`
  - `validate_variable_name()`
  - `validate_variable_value()`
  - `sanitize_for_mongodb()`
  - `validate_mongodb_field_value()`

### Changed

#### Error Handling
- Replaced critical `unwrap()` calls with proper error handling
- Improved error messages with context
- Added fallback mechanisms for non-critical failures

#### Authentication
- Added input validation to login endpoints
- Added MongoDB sanitization to username inputs
- Improved security logging for authentication events

#### Configuration
- Added CORS configuration options to `CoreConfig`
- Added environment variable support for CORS settings
- Improved configuration documentation

### Security

#### Fixed
- **CORS Security** - Changed from allowing all origins to configurable origins
- **Input Validation** - Added validation for all critical user inputs
- **Brute Force Protection** - Added rate limiting to authentication endpoints
- **NoSQL Injection** - Added input sanitization for MongoDB queries
- **Session Management** - Added clock skew tolerance to prevent token rejection

#### Improved
- **Error Handling** - Better error messages and graceful degradation
- **Security Logging** - Comprehensive logging of security events
- **Documentation** - Added security documentation and guides

---

## Migration Notes

### CORS Configuration

**Before:**
- CORS allowed all origins by default

**After:**
- CORS still allows all origins by default (backward compatible)
- Can be restricted via `cors_allowed_origins` config or environment variable

**Migration:**
```bash
# Set in environment or config file
export KOMODO_CORS_ALLOWED_ORIGINS=https://your-domain.com
```

### Input Validation

**Before:**
- Limited input validation

**After:**
- Comprehensive validation for all critical inputs

**Migration:**
- No changes required - validation is automatic
- Invalid inputs will return clearer error messages

### Rate Limiting

**Before:**
- No rate limiting on authentication endpoints

**After:**
- Rate limiting active (5 attempts / 5 minutes)

**Migration:**
- No changes required
- Users may see HTTP 429 responses if rate limit is exceeded
- Rate limit resets automatically on successful authentication

---

## Breaking Changes

None - All changes are backward compatible.

---

## Deprecations

None.

---

## Security Advisories

### CORS Configuration

**Advisory:** For production deployments, configure `cors_allowed_origins` to restrict allowed origins.

**Impact:** Low - Default behavior unchanged, but production should restrict origins.

**Recommendation:** Set `KOMODO_CORS_ALLOWED_ORIGINS` environment variable in production.

---

## References

- [Security Improvements Documentation](../SECURITY_IMPROVEMENTS.md)
- [Developer Guide](./DEVELOPER_GUIDE.md)
- [API Security Documentation](./API_SECURITY.md)

---

**For security concerns, please report them responsibly.**

