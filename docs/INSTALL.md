# Installation and verification

Use the assets on the [official release page](https://github.com/ranjanlive/indic-launchkit/releases). Choose the operating system and CPU shown in the filename and build receipt. The desktop installer and standalone CLI are separate downloads. A source checkout is also available under the MIT license.

## Verify a download

For 0.1.0-beta.1, compare its SHA-256 with SHA256SUMS-published.txt, which uses the actual GitHub download filenames. The original SHA256SUMS.txt preserves CI filenames before GitHub normalized spaces to dots. PowerShell: `Get-FileHash -Algorithm SHA256 -LiteralPath './downloaded-file'`. Linux: `sha256sum ./downloaded-file`. macOS: `shasum -a 256 ./downloaded-file`. Checksums detect changes; they are not independent publisher signatures.

## Windows x64

The setup executable installs the desktop app for the current user. Its interface requires Microsoft Edge WebView2. Extract the separate CLI ZIP and invoke launchkit.exe by an absolute path or add its folder to your own PATH. The CLI does not open a window or depend on WebView2. This beta CLI imports VCRUNTIME140.dll, so Microsoft Visual C++ v14 x64 runtime must be installed; use [Microsoft’s official supported redistributable page](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist).

## Linux x64

The native release is built on Ubuntu 22.04. The desktop app needs a graphical session and WebKitGTK 4.1. Use the Debian package with your package manager, or the AppImage on a compatible distribution. AppImages may need FUSE support; consult your distribution's documentation. Extract the CLI tar.gz separately for terminal-only operation. Linux ARM is not included in this beta.

## macOS Apple Silicon

The disk image contains the desktop app; the companion tar.gz contains the arm64 CLI. The desktop target is macOS 12+. Native build and console smoke evidence comes from macOS 15. Intel and universal binaries are not included in this beta.

## Unsigned beta

No Windows Authenticode or Apple Developer ID/notarization credentials are configured. Operating-system trust controls may restrict execution. Respect those controls; build from the public source or wait for a signed release when trusted publisher signatures are required. This project does not ask you to disable platform security.

## Remove local data

Uninstall the desktop app with the operating system's normal mechanism. Projects and exports are user-selected folders and are not automatically erased by uninstalling. Imported copies remain until you remove them. The WebView may have its normal local application-data/cache folder. CLI archives can be removed independently; preserve your project folders and releases if you still need them.
