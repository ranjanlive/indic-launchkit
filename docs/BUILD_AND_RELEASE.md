# Build and release

Build CLI and GUI separately from source_code/. Rust and npm dependencies are locked. Use Node.js 24 and stable Rust with the Tauri 2 prerequisites. The source workspace limits compilation to two workers.

Native CI runs sequentially on Windows x64, Ubuntu 22.04 x64, and macOS 15 Apple Silicon. Each runner tests the core and CLI, builds the frontend, creates its CLI archive, and builds native Tauri bundles. No macOS universal or Linux ARM binary is implied. A build is not a substitute for human usability testing on every OS.

Release automation is manually dispatched with an existing immutable tag. All native jobs must pass before the publish job can create a GitHub **pre-release**. Its binaries are unsigned: no Windows Authenticode certificate and no Apple Developer ID/notarization is configured. Follow normal operating-system safety controls. Users who require trusted signatures should build from source or wait for a signed release.

Build artifacts include platform archives, original native bundles, a SHA256SUMS.txt file, and a build receipt identifying runner OS/CPU and commit. Verify checksums after download; checksums detect file changes but are not an independent publisher signature.

For maintainers: run tests and review source/metadata first. Commit, tag, push, then dispatch the release workflow. Verify all three native receipts and assets. Stage local immutable release/<version> records using scripts/project.py; do not commit binary builds or release/private/. Update website metadata only with actual published URLs and measured sizes/hashes. Preserve earlier releases and document signing/testing limitations in release notes.
