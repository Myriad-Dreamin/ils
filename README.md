# ils

Directly read directory in image files.

**Note: it doesn't try to replicate the functionality of `ls` or `tree`.**

## Installation

Install from `crates.io`:

```bash
cargo install ils
```

Build from source:

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
