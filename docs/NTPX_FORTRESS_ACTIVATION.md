# NTPX Fortress activation

This repository is protected by the repo-local Fortress bootstrap monitor. It blocks high-confidence secrets, private-key material, sensitive credential files, and explicit L3/L4 classification markers in ordinary Git-hosted source.

## Current boundary
- Git-hosted source maximum: L2 / Confidential.
- L3 Crown Jewel and L4 Root Trust material must remain outside this repository.
- Bootstrap monitor mode is active in CI.
- Release mode remains closed until the external trust chain is activated.

## External activation still required
1. Register the repository with the fleet orchestrator.
2. Provision an external Ed25519/HSM/YubiKey/TPM/KMS signing identity.
3. Configure immutable/off-site evidence storage.
4. Enable vulnerability/container/IaC scanners appropriate to this project.
5. Enable signed release provenance/artifact attestations.
6. Configure repository rulesets and required status checks.
7. Run and record a restore/recovery drill.

Do not commit private signing keys, recovery keys, production credentials, or crown-jewel source to satisfy any Fortress check.
