# Phase 02 completion verification

Status: IN DEVELOPMENT

Candidate build: `NTPX-GFΩ-G0-v0.2.0-dev.2-M02-B000004`

## Acceptance gate

- [ ] workspace formatting passes;
- [ ] strict Clippy passes with warnings denied;
- [ ] all Rust tests pass;
- [ ] frontend typecheck/build remains green;
- [ ] full Signal Born GameSpec validates and round-trips deterministically;
- [ ] checked-in JSON Schema parses and is emitted by the CLI;
- [ ] unresolved references fail validation;
- [ ] duplicate IDs fail validation;
- [ ] dependency cycles fail validation;
- [ ] deterministic IDs are stable;
- [ ] legacy `0.1` fixture migrates to `0.1.0` and validates;
- [ ] compiler manifest is deterministic;
- [ ] CLI validation and schema commands execute successfully.

Do not mark Phase 02 complete until CI proves the final candidate.
