---
name: security-review
description: Security audit of pending code changes
---

# Security Review

Perform a security-focused review of all pending changes.

## Checklist

### Authentication & Authorization
- [ ] JWT tokens validated correctly (expiry, signature, claims)
- [ ] Passwords hashed with Argon2id (not bcrypt, not plaintext)
- [ ] No hardcoded secrets or API keys
- [ ] Bearer token required on protected endpoints
- [ ] Account active status checked during token validation

### Input Validation
- [ ] All user input validated and sanitized
- [ ] SQL injection prevention (parameterized queries via Diesel)
- [ ] No command injection in shell calls
- [ ] GraphQL query depth/complexity limits
- [ ] Request size limits enforced

### Data Protection
- [ ] No sensitive data in logs (passwords, tokens, PII)
- [ ] Error responses do not leak internal details
- [ ] No secrets in source code or config files committed to git
- [ ] Environment variables used for sensitive configuration

### Infrastructure
- [ ] Docker images use non-root users
- [ ] No unnecessary ports exposed
- [ ] Database connections use least-privilege accounts
- [ ] gRPC endpoints not exposed to public network

### Dependencies
- [ ] No known vulnerable dependencies (run `cargo audit` if available)
- [ ] Dependencies are from trusted sources
- [ ] Minimum required dependency versions specified

## Output Format

```
## Security Review Report

### Risk Level: LOW / MEDIUM / HIGH / CRITICAL

### Findings
- [CRITICAL] file:line - description + remediation
- [HIGH] file:line - description + remediation
- [MEDIUM] file:line - description + remediation
- [LOW] file:line - description + remediation

### Passed Checks
- List of checks that passed

### Recommendations
- Actionable improvements
```
