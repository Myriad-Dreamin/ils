use std::io::Write;

use clap::Parser;
use term_grid::{Alignment, Cell, Direction, Filling, Grid, GridOptions};
use unicode_width::UnicodeWidthStr;

use crate::Meta;

#[derive(Debug, Clone, Parser)]
pub struct DisplayOption {
    /// use a long listing format
    #[clap(short = 'l')]
    long: bool,
}

pub fn grid(
    flags: &DisplayOption,
    metas: Vec<Meta>,
    term_width: Option<usize>,
    out: &mut impl Write,
) -> anyhow::Result<()> {
    let mut grid = if flags.long {
        Grid::new(GridOptions {
            filling: Filling::Spaces(2),
            direction: Direction::TopToBottom,
        })
    } else {
        Grid::new(GridOptions {
            filling: Filling::Spaces(1),
            direction: Direction::LeftToRight,
        })
    };
    let mut cells = vec![];
    let mut push_cell = |cell: Cell| {
        cells.push(cell);
    };

    for meta in metas {
        if flags.long {
            let Meta {
                name,
                permissions_or_attributes,
                date,
                file_type: _,
                size,
                inode: _,
                content: _,
                owner: _,
            } = meta;

            let perm = permissions_or_attributes
                .as_ref()
                .map_or_else(|| "_".into(), |p| p.to_string());
            push_cell(Cell {
                width: get_visible_width(&perm, false),
                contents: perm,
                alignment: Alignment::Left,
            });

            let owner = meta
                .owner
                .as_ref()
                .map_or_else(|| "_".into(), |o| format!("{}:{}", o.user, o.group));
            push_cell(Cell {
                width: get_visible_width(&owner, false),
                contents: owner,
                alignment: Alignment::Left,
            });

            let size = size.map_or_else(|| "_".into(), |s| s.to_string());
            push_cell(Cell {
                width: get_visible_width(&size, false),
                contents: size,
                alignment: Alignment::Left,
            });

            let date_str = date.as_ref().map_or_else(|| "_".into(), |d| d.to_string());
            push_cell(Cell {
                width: get_visible_width(&date_str, false),
                contents: date_str,
                alignment: Alignment::Left,
            });

            let contents = name;
            push_cell(Cell {
                width: get_visible_width(&contents, false),
                contents,
                alignment: Alignment::Left,
            });
        } else {
            let contents = meta.name;
            push_cell(Cell {
                width: get_visible_width(&contents, false),
                contents,
                alignment: Alignment::Left,
            });
        };
    }

    if flags.long {
        const COLUMNS: usize = 5;

        let mut dims = [0; COLUMNS];
        for (i, cell) in cells.iter_mut().enumerate() {
            if i % COLUMNS == 0 {
                dims[0] = dims[0].max(cell.width);
            } else if i % COLUMNS == 1 {
                dims[1] = dims[1].max(cell.width);
            } else if i % COLUMNS == 2 {
                dims[2] = dims[2].max(cell.width);
            } else if i % COLUMNS == 3 {
                dims[3] = dims[3].max(cell.width);
            }
        }

        for cell in cells.chunks(COLUMNS) {
            for (i, c) in cell.iter().enumerate() {
                if i > 0 {
                    write!(out, " ")?; // two spaces between columns
                }

                write!(out, "{:width$}", c.contents, width = dims[i])?;
            }
            writeln!(out)?;
        }

        return Ok(());
    };

    for cell in cells {
        grid.add(cell);
    }

    if let Some(tw) = term_width {
        if let Some(gridded_output) = grid.fit_into_width(tw) {
            write!(out, "{gridded_output}")?;
        } else {
            //does not fit into grid, usually because (some) filename(s)
            //are longer or almost as long as term_width
            //print line by line instead!
            write!(out, "{}", grid.fit_into_columns(1))?;
        }
    } else {
        write!(out, "{}", grid.fit_into_columns(1))?;
    }

    Ok(())
}

fn get_visible_width(input: &str, hyperlink: bool) -> usize {
    let mut nb_invisible_char = 0;

    // If the input has color, do not compute the length contributed by the color to
    // the actual length
    for (idx, _) in input.match_indices("\u{1b}[") {
        let (_, s) = input.split_at(idx);

        let m_pos = s.find('m');
        if let Some(len) = m_pos {
            // len points to the 'm' character, we must include 'm' to invisible characters
            nb_invisible_char += len + 1;
        }
    }

    if hyperlink {
        for (idx, _) in input.match_indices("\x1B]8;;") {
            let (_, s) = input.split_at(idx);

            let m_pos = s.find("\x1B\x5C");
            if let Some(len) = m_pos {
                // len points to the '\x1B' character, we must include both '\x1B' and '\x5C' to
                // invisible characters
                nb_invisible_char += len + 2
            }
        }
    }

    // `UnicodeWidthStr::width` counts all unicode characters including escape
    // '\u{1b}' and hyperlink '\x1B'
    UnicodeWidthStr::width(input) - nb_invisible_char
}
