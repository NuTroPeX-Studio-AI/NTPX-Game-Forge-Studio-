# Security Policy

## Default posture

NTPX GameForge Ω∞ is offline-first and zero-trust by default.

## Protected actions

The following always require explicit human authorization:

- production publishing
- signing-key use
- credential changes
- destructive migrations or data deletion
- protected-branch merges when policy requires review
- security-policy or permission-root changes
- paid infrastructure purchases

## Agent execution principles

- least privilege
- scoped filesystem access
- scoped network access
- ephemeral credentials
- complete audit trails
- transactional edits
- snapshot/rollback before high-risk changes

## Secret handling

Never commit secrets, tokens, private keys, certificates, `.env` files, or production credentials. Use environment variables or an approved secret store.

## Reporting

Security issues should be documented privately with reproducible evidence, impact, affected components, and remediation status. Do not publish exploitable details before remediation.
