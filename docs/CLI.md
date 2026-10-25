# CLI reference · schema version 1

The `launchkit` binary is a console application separate from the desktop executable. It works without a GUI, WebView, JDK, Android SDK, or network connection. Run `launchkit --help` or `launchkit <command> --help` for exact arguments. Quote paths containing spaces.

| Command | Purpose |
|---|---|
| `init <directory> --title <text> [--package <id>] [--template aurora\|paper\|midnight]` | Create a new project directory |
| `add --project <directory> --icon <image>` | Copy and select an icon |
| `add --project <directory> --screenshot <image> [--caption <text>]` | Copy one screenshot, maximum 20 |
| `add --project <directory> --apk <file>` | Copy an APK and use its literal package/version |
| `inspect <apk>` | Read binary-manifest metadata and file checksum |
| `validate --project <directory>` | Validate all paths/assets and required export inputs |
| `preview --project <directory> --output <new.png> [--kind screenshot\|feature\|social\|icon] [--index 0]` | Render a 480px-wide preview; index is zero-based |
| `export --project <directory> --output <new-directory> [--dry-run] [--include-apk]` | Preflight or export release pack |
| `templates` | List themes |
| `schema` | Print the project JSON schema |

Add `--json` before or after the subcommand for a single JSON result on stdout. Success: `{"schema_version":1,"ok":true,"command":"...","data":...}`. Failure: `{"schema_version":1,"ok":false,"command":"operation","error":{"code":"operation_failed","message":"..."}}`. Argument failures use command `arguments` and code `invalid_arguments`. Human-readable help/version requests print their normal text.

Exit codes: **0** success/help/version, **1** validation or filesystem/operation failure, **2** invalid arguments. No prompts or implicit uploads. Messages are diagnostic; integrations should use the envelope and exit code rather than parse human text. Dry-run artifacts have zero bytes and empty hashes because no content is written; real export returns measured byte sizes and SHA-256 hashes.

APK inspection supports literal binary AndroidManifest attributes. Resource-referenced values can be null. It does not decode resource tables, verify signing certificates, identify permissions, extract app icons, or guarantee that an APK is installable. `signing_status` is always `not-verified`.

PNG, JPEG and WebP inputs only. Images are bounded to 64 MiB, 8192px per dimension, and 24 million pixels, with decoder memory limits. APK input is bounded to 1 GiB; the decompressed manifest to 8 MiB. Project assets cannot traverse paths or use symbolic links. Exports require a new output directory with an existing parent; concurrent cooperating exports reserve that name. Do not deliberately race an export with another program modifying the same directory.
