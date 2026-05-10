# Security policy

## Supported versions

Security fixes are prioritized on the default branch (`main`) and recent Bypass releases published on GitHub Releases. If you run an old version, upgrade when you can.

## Scope

Bypass is a desktop application that **reads and modifies the system hosts file** with elevated privileges when the user authorizes it. Reports we care about especially include:

- Privilege escalation, arbitrary code execution, or bypassing user confirmation outside the intended flow.
- Unexpected file read/write outside the intended scope (for example, paths not meant for app data).
- Frontend ↔ Tauri command (IPC) issues that enable abuse from untrusted content.
- Weaknesses in automatic updates or artifact integrity verification, when applicable to your build.
- Local data exfiltration or sensitive information leaks due to application bugs.

Reports that only reference known CVEs in dependencies are welcome if they include the affected versions and references (for example, Rust/npm/Tauri advisories); PRs that upgrade dependencies with official patches are also valued.

## Reporting a vulnerability

**Do not open a public issue** for unfixed vulnerabilities.

Options:

1. **GitHub Security Advisories** (recommended when enabled): https://github.com/mdiazn80/bypass → **Security** → **Report a vulnerability**.  
2. If you cannot use that channel, email or contact the repository maintainers (owner visible on GitHub, or project email if listed in the README or profile).

Include in your report:

- Technical description of the issue and its impact.  
- Steps to reproduce or a proof of concept (PoC) when safe to share.  
- Bypass version, operating system, and architecture.  
- Any temporary mitigations you know for other users.

We will respond when we can; timing depends on severity and maintainer availability.

## Coordinated disclosure

Please **do not publish vulnerability details** (blogs, social media, public issues) until a fix is released or you have explicit agreement from maintainers. We appreciate responsible disclosure efforts.

## Recognition

When appropriate, we may credit reporters in release notes or security advisories (only if they agree).
