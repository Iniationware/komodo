# API Security Documentation

**Version:** 1.0  
**Last Updated:** $(date)

---

## 🔒 Security Features Overview

Komodo implements multiple layers of security to protect the API and user data.

---

## 🔐 Authentication

### JWT Tokens

**Endpoint:** `/auth/*`

JWT tokens are used for user authentication. Tokens include:
- User ID
- Issued at timestamp (`iat`)
- Expiration timestamp (`exp`)

**Clock Skew Tolerance:** 5 minutes

**Headers:**
```
Authorization: Bearer <jwt_token>
```

### API Keys

**Headers:**
```
X-API-KEY: <api_key>
X-API-SECRET: <api_secret>
```

**Security:**
- API secrets are hashed using bcrypt
- API keys can have expiration dates
- Clock skew tolerance: 5 minutes

---

## 🚦 Rate Limiting

### Authentication Endpoints

**Rate Limit:** 5 attempts per 5 minutes per IP address

**Affected Endpoints:**
- `POST /auth/` - All authentication requests
- `POST /auth/{variant}` - Variant-specific auth requests

**Response on Rate Limit:**
```json
{
  "error": "Too many attempts. Please try again in 300 seconds.",
  "status": 429
}
```

**Behavior:**
- Rate limit is automatically reset on successful authentication
- Rate limit events are logged as security events
- IP address is extracted from `X-Forwarded-For` or `X-Real-IP` headers

---

## ✅ Input Validation

### Username Validation

**Rules:**
- Length: 1-100 characters
- Allowed: alphanumeric, underscores, hyphens, dots, @
- Cannot be a valid MongoDB ObjectId

**Endpoints:**
- `POST /auth/SignUpLocalUser`
- `POST /auth/LoginLocalUser`

### Password Validation

**Rules:**
- Minimum length: 8 characters
- Maximum length: 1000 characters
- Cannot be empty

**Endpoints:**
- `POST /auth/SignUpLocalUser`
- `POST /auth/LoginLocalUser`

### API Key Name Validation

**Rules:**
- Cannot be empty
- Maximum length: 200 characters

**Endpoints:**
- `POST /user/CreateApiKey`

---

## 🛡️ Security Headers

All API responses include the following security headers:

| Header | Value | Purpose |
|--------|-------|---------|
| `X-Content-Type-Options` | `nosniff` | Prevents MIME-type sniffing |
| `X-Frame-Options` | `DENY` | Prevents clickjacking |
| `X-XSS-Protection` | `1; mode=block` | XSS protection |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Referrer control |
| `Strict-Transport-Security` | `max-age=31536000; includeSubDomains` | HSTS (SSL only) |

---

## 📝 Security Logging

### Logged Events

Security events are logged with the following format:

```
SECURITY EVENT | type: <event_type> | ip: <ip_address> | details: <details>
```

**Event Types:**

- `failed_login` - Failed login attempt
- `failed_login_query` - Database error during login
- `rate_limit_exceeded` - Rate limit exceeded
- Successful logins are logged at INFO level

**Example:**
```
SECURITY EVENT | type: failed_login | ip: 192.168.1.1 | details: Invalid password for user: admin
```

---

## 🔒 CORS Configuration

### Default Behavior

If `cors_allowed_origins` is not configured:
- All origins are allowed (`*`)
- All methods are allowed
- All headers are allowed
- Credentials are not allowed

### Production Configuration

**Recommended:**
```toml
cors_allowed_origins = ["https://komodo.example.com"]
cors_allow_credentials = false
```

**Environment Variable:**
```bash
KOMODO_CORS_ALLOWED_ORIGINS=https://komodo.example.com,https://app.example.com
```

---

## 🚨 Error Responses

### Authentication Errors

**401 Unauthorized:**
```json
{
  "error": "must attach either AUTHORIZATION header with jwt OR pass X-API-KEY and X-API-SECRET"
}
```

**401 Unauthorized (Invalid Token):**
```json
{
  "error": "token has expired"
}
```

### Rate Limiting Errors

**429 Too Many Requests:**
```json
{
  "error": "Too many attempts. Please try again in 300 seconds."
}
```

### Validation Errors

**400 Bad Request:**
```json
{
  "error": "Invalid username format",
  "context": "Username contains invalid characters"
}
```

---

## 🔐 Best Practices

### For API Consumers

1. **Store tokens securely** - Never expose JWT tokens or API secrets
2. **Use HTTPS** - Always use HTTPS in production
3. **Handle rate limits** - Implement exponential backoff on 429 responses
4. **Validate inputs** - Validate inputs client-side before sending
5. **Monitor security events** - Monitor logs for security events

### For Developers

1. **Never log sensitive data** - Passwords, secrets, tokens should never be logged
2. **Use validation functions** - Always use validation helpers for user input
3. **Sanitize MongoDB inputs** - Use `sanitize_for_mongodb()` for user inputs
4. **Log security events** - Use `log_security_event()` for security-related events
5. **Handle errors gracefully** - Never expose internal error details to clients

---

## 📊 Security Metrics

### Current Implementation Status

| Feature | Status | Notes |
|---------|--------|-------|
| CORS Configuration | ✅ Configurable | Backward compatible |
| Security Headers | ✅ Implemented | All standard headers |
| Rate Limiting | ✅ Active | 5 attempts / 5 min |
| Input Validation | ✅ Complete | All critical endpoints |
| Security Logging | ✅ Complete | All security events |
| NoSQL Injection Protection | ✅ Implemented | Defense in depth |
| JWT Clock Skew | ✅ Implemented | 5 minute tolerance |

---

## 🔄 Migration Guide

### Updating CORS Configuration

**Before:**
```rust
// CORS allowed all origins
CorsLayer::new().allow_origin(Any)
```

**After:**
```toml
# config/core.config.toml
cors_allowed_origins = ["https://your-domain.com"]
```

**Environment Variable:**
```bash
export KOMODO_CORS_ALLOWED_ORIGINS=https://your-domain.com
```

### Adding Input Validation

**Before:**
```rust
let username = request.username;
// No validation
```

**After:**
```rust
use crate::helpers::validation::validate_username;

validate_username(&request.username)
  .context("Invalid username format")?;
```

---

## 📚 Additional Resources

- [Developer Guide](./DEVELOPER_GUIDE.md)
- [Security Improvements](../SECURITY_IMPROVEMENTS.md)
- [Code Analysis Report](../ANALYSE_BERICHT.md)

---

**For security concerns or vulnerabilities, please report them responsibly.**

