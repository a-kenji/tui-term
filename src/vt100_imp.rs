use ratatui_core::style::{Modifier, Style};

use crate::widget::{Cell, Screen};

impl Screen for vt100::Screen {
    type C = vt100::Cell;

    #[inline]
    fn cell(&self, row: u16, col: u16) -> Option<&Self::C> {
        self.cell(row, col)
    }

    #[inline]
    fn hide_cursor(&self) -> bool {
        self.hide_cursor()
    }

    #[inline]
    fn cursor_position(&self) -> (u16, u16) {
        let (row, col) = self.cursor_position();
        let scrollback = u16::try_from(self.scrollback()).unwrap_or(u16::MAX);
        (row.saturating_add(scrollback), col)
    }
}

impl Cell for vt100::Cell {
    #[inline]
    fn has_contents(&self) -> bool {
        self.has_contents()
    }

    #[inline]
    fn apply(&self, cell: &mut ratatui_core::buffer::Cell) {
        fill_buf_cell(self, cell)
    }
}

#[inline]
fn fill_buf_cell(screen_cell: &vt100::Cell, buf_cell: &mut ratatui_core::buffer::Cell) {
    let fg = screen_cell.fgcolor();
    let bg = screen_cell.bgcolor();
    if screen_cell.has_contents() {
        buf_cell.set_symbol(screen_cell.contents());
    }
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
    if screen_cell.dim() {
        style = style.add_modifier(Modifier::DIM);
    }
    buf_cell.set_style(style.fg(fg.into()).bg(bg.into()));
}

/// Represents a foreground or background color for cells.
/// Intermediate translation layer between
/// [`vt100::Screen`] and [`ratatui_core::style::Color`]
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

#[cfg(test)]
mod tests {
    use crate::widget::Screen;

    #[test]
    fn cursor_position_offset_by_scrollback() {
        let mut parser = vt100::Parser::new(6, 80, 20);
        for i in 0..9 {
            parser.process(format!("line {i}\r\n").as_bytes());
        }
        let (drawing_row, drawing_col) = parser.screen().cursor_position();

        parser.screen_mut().set_scrollback(2);
        let visible_pos = Screen::cursor_position(parser.screen());
        assert_eq!(visible_pos, (drawing_row + 2, drawing_col));
    }

    #[test]
    fn cursor_position_offset_preserves_col() {
        let mut parser = vt100::Parser::new(6, 80, 20);
        for i in 0..9 {
            parser.process(format!("line {i}\r\n").as_bytes());
        }
        parser.process(b"partial");
        let (drawing_row, drawing_col) = parser.screen().cursor_position();
        assert_eq!(drawing_col, 7);

        parser.screen_mut().set_scrollback(2);
        let (visible_row, visible_col) = Screen::cursor_position(parser.screen());
        assert_eq!(visible_row, drawing_row + 2);
        assert_eq!(visible_col, 7, "scrollback should not affect column");
    }

    #[test]
    fn cursor_position_off_screen_with_full_scrollback() {
        let mut parser = vt100::Parser::new(4, 80, 20);
        for i in 0..10 {
            parser.process(format!("line {i}\r\n").as_bytes());
        }

        parser.screen_mut().set_scrollback(4);
        let (visible_row, _) = Screen::cursor_position(parser.screen());
        assert!(
            visible_row >= 4,
            "cursor at visible row {visible_row} should be off-screen (>= 4 rows)"
        );
    }
}

impl From<Color> for ratatui_core::style::Color {
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
