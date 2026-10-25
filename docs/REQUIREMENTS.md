# Indic LaunchKit — first public release

An offline, open-source release preparation workbench for independent app developers. The desktop interface and headless CLI must use the same engine and export format. No account, telemetry, AI service, web upload, signing service or subscription is required.

## Acceptance criteria

- Windows, Linux and macOS desktop builds, plus standalone native CLI archives. Test each platform on its own CI runner. Declare architecture and signing/notarization state honestly.
- Inspect a real APK without executing it: package, version name/code, minimum/target SDK, byte size and SHA-256. Inspection does not verify the APK signature or certify the app.
- Persist a versioned, documented project JSON with local icon, screenshots, captions and optional APK. Unknown fields, unsafe paths and invalid assets fail before export.
- Three original templates: Aurora, Paper and Midnight. Preview and export use the same Rust SVG renderer and bundled redistributable fonts.
- Export PNG screenshot compositions, a 512px icon, feature graphic, social card, release metadata and SHA256SUMS. A selected APK may be included explicitly. Never invent store links, ratings or compliant privacy statements.
- CLI supports init, add, inspect, validate, preview, export, templates and schema; --json produces one machine-readable response and documented exit codes. No prompts or GUI startup for CLI commands. Dry run writes nothing.
- Output publication is transactional, refuses existing destinations, has bounded memory/input sizes and does not fetch remote resources. No arbitrary SVG input or untrusted HTML execution.
- Desktop UI supports a complete create/import/design/preview/export path, keyboard use, clear errors and operation state. Stop owned preview/build/app helpers after testing.
- Public repository includes a deliberate open-source license, contribution/security guidance, reproducible locked dependency installs, native CI builds and a prerelease with checksums.

## Boundaries

The first release is a beta, not a full free-form graphics editor. It does not sign binaries, upload to app stores, change app interfaces, validate legal compliance, remove watermarks from third-party artwork, or promise store acceptance. No Windows/macOS signing credentials are provisioned. Unsupported scripts or missing font glyphs must be handled explicitly rather than silently rendered as good output.
