use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::widget::PseudoTerm;

/// Draw the [`Screen`] to the [`Buffer`],
/// area is the designated area that the consumer provides
pub fn handle(term: &PseudoTerm, area: &Rect, buf: &mut Buffer) {
    let cols = area.width;
    let rows = area.height;
    let col_start = area.x;
    let row_start = area.y;
    let area_cols = area.width + area.x;
    let area_rows = area.height + area.y;
    let screen = term.screen();

    // The [`Screen`] is made out of rows of cells
    for row in 0..rows {
        for col in 0..cols {
            let buf_col = col + col_start;
            let buf_row = row + row_start;

            if buf_row > area_rows || buf_col > area_cols {
                // Skip writing outside the area
                continue;
            }

            if let Some(screen_cell) = screen.cell(row, col) {
                if screen_cell.has_contents() {
                    let fg = screen_cell.fgcolor();
                    let bg = screen_cell.bgcolor();

                    let cell = buf.get_mut(buf_col, buf_row);
                    cell.set_symbol(&screen_cell.contents());
                    let fg: Color = fg.into();
                    let bg: Color = bg.into();
                    let mut style = Style::reset();
                    if screen_cell.bold() {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    if screen_cell.italic() {
                        style = style.add_modifier(Modifier::ITALIC);
                    }
                    if screen_cell.underline() {
                        style = style.add_modifier(Modifier::UNDERLINED);
                    }
                    if screen_cell.inverse() {
                        style = style.add_modifier(Modifier::REVERSED);
                    }
                    cell.set_style(style);
                    cell.set_fg(fg.into());
                    cell.set_bg(bg.into());
                }
            }
        }
    }

    if !screen.hide_cursor() {
        let (c_row, c_col) = screen.cursor_position();
        if c_row < area_rows && c_col < area_cols {
            let c_cell = buf.get_mut(c_col + col_start, c_row + row_start);
            if let Some(cell) = screen.cell(c_row, c_col) {
                if cell.has_contents() {
                    let style = term.cursor.overlay_style;
                    c_cell.set_style(style);
                } else {
                    let symbol = &term.cursor.symbol;
                    let style = term.cursor.style;
                    c_cell.set_symbol(symbol);
                    c_cell.set_style(style);
                }
            }
        }
    }
}

/// Represents a foreground or background color for cells.
/// Intermediate translation layer between
/// [`vt100::Screen`] and [`ratatui::style::Color`]
#[allow(dead_code)]
enum Color {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
    Rgb(u8, u8, u8),
    Indexed(u8),
}

impl From<vt100::Color> for Color {
    fn from(value: vt100::Color) -> Self {
        match value {
            vt100::Color::Default => Color::Reset,
            vt100::Color::Idx(i) => Color::Indexed(i),
            vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
        }
    }
}

impl From<Color> for vt100::Color {
    fn from(value: Color) -> Self {
        match value {
            Color::Reset => vt100::Color::Default,
            Color::Black => vt100::Color::Idx(0),
            Color::Red => vt100::Color::Idx(1),
            Color::Green => vt100::Color::Idx(2),
            Color::Yellow => vt100::Color::Idx(3),
            Color::Blue => vt100::Color::Idx(4),
            Color::Magenta => vt100::Color::Idx(5),
            Color::Cyan => vt100::Color::Idx(6),
            Color::Gray => vt100::Color::Idx(7),
            Color::DarkGray => vt100::Color::Idx(8),
            Color::LightRed => vt100::Color::Idx(9),
            Color::LightGreen => vt100::Color::Idx(10),
            Color::LightYellow => vt100::Color::Idx(11),
            Color::LightBlue => vt100::Color::Idx(12),
            Color::LightMagenta => vt100::Color::Idx(13),
            Color::LightCyan => vt100::Color::Idx(14),
            Color::White => vt100::Color::Idx(15),
            Color::Rgb(r, g, b) => vt100::Color::Rgb(r, g, b),
            Color::Indexed(i) => vt100::Color::Idx(i),
        }
    }
}

impl From<Color> for ratatui::style::Color {
    fn from(value: Color) -> Self {
        match value {
            Color::Reset => ratatui::style::Color::Reset,
            Color::Black => ratatui::style::Color::Black,
            Color::Red => ratatui::style::Color::Red,
            Color::Green => ratatui::style::Color::Green,
            Color::Yellow => ratatui::style::Color::Yellow,
            Color::Blue => ratatui::style::Color::Blue,
            Color::Magenta => ratatui::style::Color::Magenta,
            Color::Cyan => ratatui::style::Color::Cyan,
            Color::Gray => ratatui::style::Color::Gray,
            Color::DarkGray => ratatui::style::Color::DarkGray,
            Color::LightRed => ratatui::style::Color::LightRed,
            Color::LightGreen => ratatui::style::Color::LightGreen,
            Color::LightYellow => ratatui::style::Color::LightYellow,
            Color::LightBlue => ratatui::style::Color::LightBlue,
            Color::LightMagenta => ratatui::style::Color::LightMagenta,
            Color::LightCyan => ratatui::style::Color::LightCyan,
            Color::White => ratatui::style::Color::White,
            Color::Rgb(r, g, b) => ratatui::style::Color::Rgb(r, g, b),
            Color::Indexed(i) => ratatui::style::Color::Indexed(i),
        }
    }
}
