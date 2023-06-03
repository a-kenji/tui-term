use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color as RatatuiColor;

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

impl From<Color> for RatatuiColor {
    fn from(value: Color) -> Self {
        match value {
            Color::Reset => RatatuiColor::Reset,
            Color::Black => RatatuiColor::Black,
            Color::Red => RatatuiColor::Red,
            Color::Green => RatatuiColor::Green,
            Color::Yellow => RatatuiColor::Yellow,
            Color::Blue => RatatuiColor::Blue,
            Color::Magenta => RatatuiColor::Magenta,
            Color::Cyan => RatatuiColor::Cyan,
            Color::Gray => RatatuiColor::Gray,
            Color::DarkGray => RatatuiColor::DarkGray,
            Color::LightRed => RatatuiColor::LightRed,
            Color::LightGreen => RatatuiColor::LightGreen,
            Color::LightYellow => RatatuiColor::LightYellow,
            Color::LightBlue => RatatuiColor::LightBlue,
            Color::LightMagenta => RatatuiColor::LightMagenta,
            Color::LightCyan => RatatuiColor::LightCyan,
            Color::White => RatatuiColor::White,
            Color::Rgb(r, g, b) => RatatuiColor::Rgb(r, g, b),
            Color::Indexed(i) => RatatuiColor::Indexed(i),
        }
    }
}
