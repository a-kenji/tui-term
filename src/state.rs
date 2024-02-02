use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
};

use crate::widget::PseudoTerminal;

/// Draw the [`Screen`] to the [`Buffer`],
/// area is the designated area that the consumer provides
pub fn handle(term: &PseudoTerminal, area: Rect, buf: &mut Buffer) {
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

    if !screen.hide_cursor() && term.cursor.show {
        let (c_row, c_col) = screen.cursor_position();
        if (c_row + row_start) < area_rows && (c_col + col_start) < area_cols {
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
    #[inline]
    fn from(value: vt100::Color) -> Self {
        match value {
            vt100::Color::Default => Self::Reset,
            vt100::Color::Idx(i) => Self::Indexed(i),
            vt100::Color::Rgb(r, g, b) => Self::Rgb(r, g, b),
        }
    }
}

impl From<Color> for vt100::Color {
    #[inline]
    fn from(value: Color) -> Self {
        match value {
            Color::Reset => Self::Default,
            Color::Black => Self::Idx(0),
            Color::Red => Self::Idx(1),
            Color::Green => Self::Idx(2),
            Color::Yellow => Self::Idx(3),
            Color::Blue => Self::Idx(4),
            Color::Magenta => Self::Idx(5),
            Color::Cyan => Self::Idx(6),
            Color::Gray => Self::Idx(7),
            Color::DarkGray => Self::Idx(8),
            Color::LightRed => Self::Idx(9),
            Color::LightGreen => Self::Idx(10),
            Color::LightYellow => Self::Idx(11),
            Color::LightBlue => Self::Idx(12),
            Color::LightMagenta => Self::Idx(13),
            Color::LightCyan => Self::Idx(14),
            Color::White => Self::Idx(15),
            Color::Rgb(r, g, b) => Self::Rgb(r, g, b),
            Color::Indexed(i) => Self::Idx(i),
        }
    }
}

impl From<Color> for ratatui::style::Color {
    #[inline]
    fn from(value: Color) -> Self {
        match value {
            Color::Reset => Self::Reset,
            Color::Black => Self::Black,
            Color::Red => Self::Red,
            Color::Green => Self::Green,
            Color::Yellow => Self::Yellow,
            Color::Blue => Self::Blue,
            Color::Magenta => Self::Magenta,
            Color::Cyan => Self::Cyan,
            Color::Gray => Self::Gray,
            Color::DarkGray => Self::DarkGray,
            Color::LightRed => Self::LightRed,
            Color::LightGreen => Self::LightGreen,
            Color::LightYellow => Self::LightYellow,
            Color::LightBlue => Self::LightBlue,
            Color::LightMagenta => Self::LightMagenta,
            Color::LightCyan => Self::LightCyan,
            Color::White => Self::White,
            Color::Rgb(r, g, b) => Self::Rgb(r, g, b),
            Color::Indexed(i) => Self::Indexed(i),
        }
    }
}
