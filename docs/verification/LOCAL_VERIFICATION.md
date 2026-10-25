# Local verification · first beta

- Rust core: malformed/truncated APK chunks, relative-path rejection, unknown project fields, missing assets, unsupported captions, real render dimensions, dry-run/no overwrite, export hashes, and all themes with Devanagari text. Six Windows tests passed. The Unix symlink case additionally runs on Linux/macOS CI.
- Built standalone Windows x64 CLI. scripts/smoke_cli.py passed seven end-to-end checks using synthetic images and a minimal valid binary Android manifest: inspect, init/import, validate, preflight, real export including optional APK, checksums, preview, and failure codes.
- Inspected a real IndicStudio AnyReader debug APK: package com.indicstudio.anyreader, versionName 1.1.0, versionCode 4, min SDK 28, target SDK 37, 85,028,478 bytes. SHA-256 matched its existing release record: 25fbe18a4e098f08f19add64c3d80f2fa3cccfd7934ac0ae8346ad8d29c723e4. Signature status remained not-verified.
- Windows Tauri WebView integration passed seven checks with actual Rust IPC. Only native file-picker selections were injected in the development build; production builds remove that injection. Real company screenshots were imported for the public demo. See desktop-smoke.json. Native file-picker usability was not fully automated.
- A Windows x64 NSIS installer was built locally. Source changes after that initial build are rebuilt by the tagged native release workflow. The published release's runner receipts identify its exact source commit and checksums.
- Linux/macOS interactive GUI testing is not claimed. Their native compilation, core tests and console smoke evidence come from their respective CI runners.
- Local temporary filesystem tests required host execution because the sandbox denied temporary-file publication. Host tests passed; the sandbox error was not suppressed or counted as a product success.

No Authenticode or Apple notarization credentials were supplied. No application store submission, APK installation, signature verification or signing was performed by this tool.
