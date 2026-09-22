# Security Policy

## Supported Versions

The following versions are currently being supported with security updates:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | ✅ Active           |
| < 0.1.0 | ❌ Unsupported      |

## Reporting a Vulnerability

We take the security of SteelSeries GG for Linux seriously. If you believe you have found a security vulnerability, please report it immediately following these guidelines:

### How to Report

**DO NOT** create public issues for security vulnerabilities. Instead, please use one of the following channels:

1. **Email**: security@steelseries-linux.dev (PGP Key available below)
2. **GitHub Security Advisory Program**: https://github.com/MikhaelCat/SteelSeries-GG-for-linux/security/advisories/new

### What to Include

When reporting a security issue, please provide:

- A descriptive summary of the vulnerability
- Steps to reproduce the issue
- Affected versions and configurations
- Proof-of-concept code or test cases (if applicable)
- Potential impact assessment
- Your contact information for follow-up questions

### Response Timeline

We commit to:

1. Acknowledging receipt within 48 hours
2. Providing initial assessment within 5 business days
3. Delivering a fix or mitigation plan within 30 days
4. Publishing CVE assignment if applicable

## Security Features

This project implements the following security measures:

### Hardware Access Control
- Device permissions via udev rules (UID/GID restrictions)
- No root privileges required for normal operation
- Isolated systemd service execution context

### Data Protection
- Configuration files stored in user home directory
- No telemetry or data collection
- Local-only network binding (127.0.0.1) for GameSense API

### Build Security
- Reproducible builds supported
- Dependency vulnerability scanning via cargo-audit
- Memory safety through Rust type system

## Security Best Practices

### Installation
- Always verify package signatures before installation
- Use official distribution packages when available
- Run `scripts/security-audit.sh` after installation

### Runtime
- Regularly update to latest version
- Monitor journal logs for unusual activity
- Review device access logs periodically

### Development
- Keep dependencies updated via `cargo update`
- Run `cargo audit` before each release
- Enable all compiler warnings (`-D warnings`)

## PGP Key

For encrypted communications, please use our PGP key:

```
-----BEGIN PGP PUBLIC KEY BLOCK-----
Version: OpenPGP.js v4.10.10

mQINBGAxMk... [Truncated for brevity]
...END-PGP PUBLIC KEY BLOCK-----
```

Note: Full key available at `assets/secring.gpg` or via secure channel.

## Credits

Security testing and bug reports from the community are greatly appreciated. Thank you to all contributors who help make this project more secure.

---

Last updated: September 2026
