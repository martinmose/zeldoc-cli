use std::io::{self, Write};

pub enum Align {
    Left,
    Right,
}

/// Rows under a header as aligned columns, two spaces apart, without
/// trailing spaces.
pub fn write_table<const N: usize>(
    out: &mut impl Write,
    columns: &[(&str, Align); N],
    rows: &[[String; N]],
) -> io::Result<()> {
    let header = columns.each_ref().map(|(title, _)| title.to_string());
    let mut widths = [0; N];
    for row in std::iter::once(&header).chain(rows) {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.chars().count());
        }
    }

    for row in std::iter::once(&header).chain(rows) {
        let cells: Vec<String> = row
            .iter()
            .zip(&widths)
            .zip(columns)
            .map(|((cell, &width), (_, align))| match align {
                Align::Left => format!("{cell:<width$}"),
                Align::Right => format!("{cell:>width$}"),
            })
            .collect();
        writeln!(out, "{}", cells.join("  ").trim_end())?;
    }
    Ok(())
}
