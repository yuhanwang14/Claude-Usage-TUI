# Security Policy

## Reporting a Vulnerability

**Do NOT open a public issue for security vulnerabilities.**

Please report security issues by opening a [GitHub Security Advisory](https://github.com/yuhanwang14/claude-usage-tui/security/advisories/new).

Include:
- Description of the vulnerability
- Steps to reproduce
- Impact assessment

## What to Expect

- Acknowledgment within 1 week
- Fix before public disclosure
- Credit in the advisory (unless you decline)

## Scope

This project handles OAuth tokens and session cookies locally. Security concerns include:
- Credential storage and handling
- Network requests to claude.ai and api.anthropic.com
- Terminal state cleanup (no credential leakage on exit)

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | Yes       |
