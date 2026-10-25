# Indic LaunchKit implementation and release plan

## Architecture

Rust workspace in source_code/: launchkit-core (project model, bounded APK parser, image validation, templates and transactional export), launchkit-cli (clap and stable JSON envelopes), and src-tauri (small native desktop adapter). A TypeScript/Vite interface calls the engine through narrowly scoped Tauri commands. The CLI does not link the webview or open a desktop window.

The renderer uses SVG generated from trusted templates, embedded raster images and bundled licensed fonts. It resolves no remote URLs. It reads project-relative assets and rejects traversal/symlinks. Export is sequential and staged in a fresh sibling directory, then atomically renamed; an existing target is never overwritten. App-signature verification and legal notices remain outside scope.

## Milestones

1. Establish workspace, schema, CLI contract, license decision and public repository ownership.
2. Implement inspection/project validation and rendering/export with focused boundary and real-APK tests.
3. Build a polished desktop workflow: project details, asset import, template selection, caption editing, preview and export. Generate a product-specific icon and capture the real app.
4. Verify CLI end-to-end and GUI locally on Windows. Build/test Windows x64, Linux x64 and macOS arm64 on native GitHub runners, with two compilation jobs and max-parallel 1.
5. Create immutable 0.1.0-beta.1 artifacts, manifests and checksums. Review public source for private/generated files. Publish the open-source repository and GitHub prerelease after native build verification.
6. Add only verified downloads to the product page, with accurate platform/signing/test limitations, privacy notice, CLI examples and release notes. Publish the updated website.

## CLI example

```sh
launchkit init my-app --title "My App" --package com.example.myapp --json
launchkit add --project my-app --icon icon.png --json
launchkit add --project my-app --screenshot home.png --caption "Your day, organized" --json
launchkit inspect app.apk --json
launchkit validate --project my-app --json
launchkit export --project my-app --output ./exports/my-app-0.1.0 --dry-run --json
launchkit export --project my-app --output ./exports/my-app-0.1.0 --json
```

JSON responses carry schema_version, ok, command and data or a structured error. Success is exit 0; invalid arguments/project/assets and I/O failures are nonzero. No hidden reasoning, credentials or provider data is part of the product.

## Evidence and release policy

Use docs/verification for bounded test/build/smoke summaries. Generated caches, raw logs, local projects, native binaries and signing material are excluded from Git. CLI and GUI are tested separately; successful builds do not imply an interactive GUI smoke test occurred on an unavailable operating system. macOS notarization and Windows Authenticode remain unverified unless genuine credentials are later supplied. Keep signing fields explicit in release metadata.
