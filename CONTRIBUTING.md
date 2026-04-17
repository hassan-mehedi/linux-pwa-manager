# Contributing

## Setup

1. Install the Linux system dependencies listed in [README.md](./README.md).
2. Install project dependencies with `npm install`.
3. Run checks before opening a change:

```bash
npm run check:ts
npm run fmt:check
npm run lint:rust
npm test
cargo test --manifest-path src-tauri/Cargo.toml
```

## Expectations

- Keep frontend and backend payloads in sync.
- Add or update tests for user-facing behavior changes.
- Avoid destructive migration changes without an explicit migration path.
- Preserve user data under the managed XDG directories.
