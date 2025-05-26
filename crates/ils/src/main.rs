//! # ils
//!
//! Directly read directory in image files.
//!
//! ## Example
//!
//! Reads root directory entries either from a block device:
//!
//! ```bash
//! ils -f /dev/sda1
//! ```
//!
//! Or from an image file:
//!
//! ```bash
//! ils -f ext4.img
//! ```
//!
//! Reads a subpath, e.g. `/home/user`:
//!
//! ```bash
//! ils -f /dev/sda1 --subpath /home/user
//! ```

use std::io::Write;
use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use path_clean::PathClean;

#[derive(Debug, Clone, Parser)]
struct Opts {
    /// The file to read from, e.g. `/dev/sda1` or `ext4.img`.
    #[clap(long, short = 'f', value_name = "FILE")]
    from: String,
    /// The subpath to read from, e.g. `/home/user`.
    #[clap(default_value = "/")]
    subpath: String,
    // fs type
    // source file
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();
    let source = opts.from;

    let subpath = PathBuf::from("/").join(opts.subpath).clean();
    let subpath = subpath.to_str().expect("Invalid subpath");

    let path = std::path::PathBuf::from(source);
    let file = std::fs::File::open(&path)
        .with_context(|| format!("Failed to open file: {}", path.display()))?;

    let super_block = ext4::SuperBlock::new(&file).context("Failed to create superblock")?;

    let dirent = super_block
        .resolve_path(subpath)
        .context("Failed to resolve path")?;
    let reading = super_block
        .load_inode(dirent.inode)
        .context("Failed to get inode")?;

    let mut files = vec![];
    let mut is_dir = None;
    super_block.walk(&reading, subpath, &mut |_, _, _, enhanced| {
        match enhanced {
            ext4::Enhanced::SymbolicLink(..) => {
                return Ok(true);
            }
            ext4::Enhanced::CharacterDevice(_, _)
            | ext4::Enhanced::BlockDevice(_, _)
            | ext4::Enhanced::Fifo
            | ext4::Enhanced::Socket
            | ext4::Enhanced::RegularFile => {
                is_dir = Some(false);
            }
            ext4::Enhanced::Directory(items) => {
                is_dir = Some(true);
                for item in items {
                    if item.name == "." || item.name == ".." {
                        continue;
                    }

                    files.push(item.name.to_string());
                }
            }
        }

        Ok(false)
    })?;

    if !is_dir.unwrap_or(false) {
        anyhow::bail!("Path is not a directory");
    }

    let mut stdout = std::io::stdout().lock();
    for file in files {
        writeln!(stdout, "{file}")?;
    }

    Ok(())
}
