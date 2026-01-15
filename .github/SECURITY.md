# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security issue, please report it responsibly.

### How to Report

1. **Do NOT** open a public GitHub issue for security vulnerabilities
2. Email security concerns to the maintainers privately
3. Include as much detail as possible:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### What to Expect

- **Acknowledgment**: Within 48 hours of your report
- **Initial Assessment**: Within 7 days
- **Resolution Timeline**: Depends on severity
  - Critical: 7 days
  - High: 14 days
  - Medium: 30 days
  - Low: 90 days

### Disclosure Policy

- We follow coordinated disclosure practices
- Security advisories will be published via GitHub Security Advisories
- Credit will be given to reporters (unless anonymity is requested)

## Security Measures

### Dependency Auditing

We use `cargo-deny` to audit dependencies for:
- Known vulnerabilities (RustSec Advisory Database)
- Unmaintained crates
- License compliance

### Code Security

- No unsafe code in core library (enforced via `#![forbid(unsafe_code)]` where applicable)
- All PRs require passing security audit
- Dependencies are regularly updated

### CI/CD Security

- Trusted publishing for releases (no stored credentials)
- Minimal permissions in GitHub Actions workflows
- Dependency caching does not include sensitive data

## Scope

This security policy applies to:
- `scrape-core` (Rust library)
- `scrape-py` (Python bindings)
- `scrape-rs` (Node.js bindings)
- `@scrape-rs/wasm` (WebAssembly bindings)

## Out of Scope

- Vulnerabilities in dependencies (report upstream)
- Issues in example code or documentation
- Theoretical attacks without proof of concept
