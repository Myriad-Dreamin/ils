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
//! ils -f /dev/sda1 /home/user
//! ```

use std::path::PathBuf;

use anyhow::Context;
use chrono::DateTime;
use clap::Parser;
use display::{DisplayOption, grid};
use path_clean::PathClean;

mod display;
mod meta;
use meta::*;

#[derive(Debug, Clone, Parser)]
struct Opts {
    /// The file to read from, e.g. `/dev/sda1` or `ext4.img`.
    #[clap(long, short = 'f', value_name = "FILE")]
    from: String,
    /// The subpath to read from, e.g. `/home/user`.
    #[clap(default_value = "/")]
    subpath: String,

    #[clap(flatten)]
    display: DisplayOption,
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
    super_block.walk(&reading, subpath, &mut |_, _, inode, enhanced| {
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

                    let file_mode = inode.stat.file_mode;

                    let file_type = match item.file_type {
                        ext4::FileType::RegularFile => FileType::File {
                            uid: false,
                            exec: false,
                        },
                        ext4::FileType::Directory => FileType::Directory { uid: false },
                        ext4::FileType::SymbolicLink => FileType::SymLink { is_dir: false },
                        ext4::FileType::CharacterDevice => FileType::CharDevice,
                        ext4::FileType::BlockDevice => FileType::BlockDevice,
                        ext4::FileType::Fifo => FileType::Pipe,
                        ext4::FileType::Socket => FileType::Socket,
                    };

                    files.push(Meta {
                        name: item.name.to_string(),
                        permissions_or_attributes: Some(PermissionsOrAttributes::Permissions(
                            Permissions::from_mode(file_mode),
                        )),
                        date: {
                            let mut sys_t = &inode.stat.mtime;
                            if sys_t.epoch_secs == 0 && sys_t.nanos.is_none() {
                                sys_t = &inode.stat.ctime;
                            }

                            let u = DateTime::from_timestamp(
                                sys_t.epoch_secs as i64,
                                0,
                                // todo: > 1e9
                                // sys_t.nanos.unwrap_or_default()
                            );
                            u.map(|date| date.into()).map(Date::Date)
                        },
                        owner: Some(Owner {
                            user: inode.stat.uid,
                            group: inode.stat.gid,
                        }),
                        file_type,
                        size: Some(inode.stat.size),
                        inode: Some(item.inode as u64),
                        content: None,
                    });
                }
            }
        }

        Ok(false)
    })?;

    if !is_dir.unwrap_or(false) {
        anyhow::bail!("Path is not a directory");
    }

    let term_width = terminal_size::terminal_size().map(|(w, _)| w.0 as usize);

    let mut stdout = std::io::stdout().lock();
    grid(&opts.display, files, term_width, &mut stdout).context("Failed to display grid")?;

    Ok(())
}
