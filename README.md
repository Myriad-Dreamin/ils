# ils

Directly read directory in image files.

**Your data is important. Although `ils` never writes to the filesystem, it is still recommended to make a backup of your data before using this tool. `ils` will not take responsibility for any data loss.**

**Note: it doesn't try to replicate the functionality of `ls` or `tree`.**

## Installation

Check [GitHub Releases.](https://github.com/Myriad-Dreamin/ils/releases)

Or install from `crates.io`:

```bash
cargo install ils
```

You can also build from source:

```bash
cargo install --path crates/ils
```

## Usage

Reads root directory entries either from a block device:

```bash
ils -f /dev/sda1
```

Or from an image file:

```bash
ils -f ext4.img
```

Reads a subpath, e.g. `/home/user`:

```bash
ils -f /dev/sda1 --subpath /home/user
```

## Development

VSCode:

```bash
code .vscode/ils.code-workspace
```

## Why not `binwalk`?

[`binwalk`](https://github.com/ReFirmLabs/binwalk) is awesome, but it is not easy to use `binwalk`. Furthermore, and it extracts files by shell calls.

## Disclaimer

Your data is important. Although `ils` never writes to the filesystem, it is still recommended to make a backup of your data before using this tool. `ils` will not take responsibility for any data loss.
