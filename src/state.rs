use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::widget::PseudoTerm;

/// Draw the [`Screen`] to the [`Buffer`],
/// area is the designated area that the consumer provides
pub fn handle(term: &PseudoTerm, area: &Rect, buf: &mut Buffer) {
    let cols = area.width;
    let rows = area.height;
    let col_start = area.x;
    let row_start = area.y;
    let screen = term.screen();

    // The [`Screen`] is made out of rows of cells
    for row in 0..rows {
        for col in 0..cols {
            if let Some(screen_cell) = screen.cell(row, col) {
                if screen_cell.has_contents() {
                    let fg = screen_cell.fgcolor();
                    let bg = screen_cell.bgcolor();

                    let cell = buf.get_mut(col + col_start, row + row_start);
                    cell.set_symbol(&screen_cell.contents());
                    let fg: Color = fg.into();
                    cell.set_fg(fg.into());
                    let bg: Color = bg.into();
                    cell.set_bg(bg.into());
                }
            }
        }
    }

    if !screen.hide_cursor() {
        let (c_col, c_row) = screen.cursor_position();
        let c_cell = buf.get_mut(c_col + col_start, c_row + row_start);
        c_cell.set_symbol("█");
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
            Color::Black => vt100::Color::Default,
            Color::Red => vt100::Color::Default,
            Color::Green => vt100::Color::Default,
            Color::Yellow => vt100::Color::Default,
            Color::Blue => vt100::Color::Default,
            Color::Magenta => vt100::Color::Default,
            Color::Cyan => vt100::Color::Default,
            Color::Gray => vt100::Color::Default,
            Color::DarkGray => vt100::Color::Default,
            Color::LightRed => vt100::Color::Default,
            Color::LightGreen => vt100::Color::Default,
            Color::LightYellow => vt100::Color::Default,
            Color::LightBlue => vt100::Color::Default,
            Color::LightMagenta => vt100::Color::Default,
            Color::LightCyan => vt100::Color::Default,
            Color::White => vt100::Color::Default,
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
