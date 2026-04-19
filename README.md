# nfc2nfd

Rename files from NFC (Windows/Linux default) to NFD (Apple default) so the iOS `UIDocumentPicker` can select CJK-named files.

## Why

Windows and Linux store filenames in NFC (precomposed) form. Apple platforms use NFD (decomposed). When you copy CJK-named files to an iOS device via USB or a file server, the system document picker may fail to select them because the normalization form doesn't match what iOS expects. This tool batch-renames filenames to NFD, fixing the problem.

## Install

```
cargo install nfc2nfd
```

Or download a prebuilt binary from [Releases](https://github.com/ooopus/nfc2nfd/releases).

## Usage

```
nfc2nfd [OPTIONS] [PATH]
```

| Flag | Description |
|------|-------------|
| `-n, --dry-run` | Print planned renames without touching the filesystem |
| `-r, --recursive` | Recurse into subdirectories |
| `-D, --include-dirs` | Also rename directory names (default: files only) |
| `-v, --verbose` | Print entries that are already NFD |

### Examples

Preview what would be renamed:

```
nfc2nfd -rn /path/to/files
```

Rename all files recursively, including directories:

```
nfc2nfd -rD /path/to/files
```

## License

MIT
