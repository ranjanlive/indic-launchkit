# Portable projects

A project is a directory containing launchkit.json and imported assets/. Paths in the JSON are relative, slash-separated, and confined to the project. Imported files use content-derived names. Removing an asset from the project list does not erase the original or previously imported file.

```json
{
  "schema_version": 1,
  "title": "Little Steps",
  "tagline": "A calmer way to organize your day",
  "package_name": "com.example.steps",
  "version": "1.0.0",
  "template": "aurora",
  "icon": "assets/icon-example.png",
  "apk": null,
  "screenshots": [{"file":"assets/screenshot-example.png","caption":"Make room for progress"}]
}
```

Run `launchkit schema --json` to obtain the machine-readable schema. Unknown fields are rejected. Titles have 1–64 characters, taglines up to 140, captions up to 180, and at most 20 screenshots. Template layouts impose additional visual limits; export preflight reports text that must be shortened. Bundled fonts cover Latin and Devanagari; unsupported characters produce an explicit error. This release uses curated layouts, not a freeform editor or automatic translation service.

Metadata is written through a temporary file in the same directory. Export rendering happens in a temporary sibling directory and is published on success. Images, metadata, and checklist have a SHA256SUMS.txt record; the checksum file does not recursively hash itself. The receipt also provides its checksum.

The project is not a secret vault. Do not import confidential screenshots or APKs into a directory that you publish. Export packs may include product metadata and images; APK copying requires --include-apk or its desktop checkbox.
