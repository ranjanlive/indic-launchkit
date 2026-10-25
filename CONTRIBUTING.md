# Contributing

Open an issue for a proposed feature or submit a focused pull request. Keep the desktop and CLI behavior consistent through launchkit-core. Preserve bounded decoding, relative project assets, no network access in the application, and refusal to overwrite exports.

Before submitting: run `cargo fmt --all --check`, `cargo test -p launchkit-core -p launchkit-cli --locked`, `npm ci`, `npm run build` from source_code, then `python scripts/project.py validate` from the repository root. Native builds need the Tauri platform prerequisites.

Add meaningful tests for behavioral changes. Do not commit generated targets, installers, private keys, personal screenshots, local project paths, or caches. Use synthetic assets for tests. Contributions are submitted under the repository's MIT license. Report security issues according to SECURITY.md.
