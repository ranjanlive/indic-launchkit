Indic LaunchKit's first public beta: an offline desktop studio and companion CLI sharing a Rust engine.

Create portable projects, import real icons/screenshots, inspect APK metadata, choose three visual themes, and export checksummed artwork packs. CLI supports noninteractive arguments, --json, --dry-run, project validation and a machine-readable schema.

Downloads include native desktop bundles and separate CLI archives for Windows x64, Linux x64 and macOS Apple Silicon. Build receipts and CLI smoke reports identify the actual runner and source commit. Verify SHA256SUMS.txt after download.

**Unsigned beta:** Windows Authenticode signing and Apple Developer ID/notarization are not configured. Respect your operating system's security controls. Source builds are available. Linux requires a compatible desktop/WebKitGTK 4.1 environment; CLI needs no graphical desktop. Intel macOS and Linux ARM are not included in this beta.

APK inspection reads literal binary manifest fields; it does not verify signatures or guarantee installability. Store acceptance, privacy policies, legal review, signing and publication remain separate steps. Text is supported by bundled Noto Latin/Devanagari fonts; unsupported characters or text that exceeds template limits produce explicit errors.

No analytics, accounts or application network calls. Selected files and generated exports stay on your device. MIT source; fonts retain SIL OFL 1.1.
