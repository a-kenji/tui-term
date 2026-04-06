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
    if screen_cell.has_contents() {
        buf_cell.set_symbol(screen_cell.contents());
    }

    let mut modifier = Modifier::empty();
    if screen_cell.bold() {
        modifier |= Modifier::BOLD;
    }
    if screen_cell.italic() {
        modifier |= Modifier::ITALIC;
    }
    if screen_cell.underline() {
        modifier |= Modifier::UNDERLINED;
    }
    if screen_cell.inverse() {
        modifier |= Modifier::REVERSED;
    }
    if screen_cell.dim() {
        modifier |= Modifier::DIM;
    }

    let fg = map_color(screen_cell.fgcolor());
    let bg = map_color(screen_cell.bgcolor());

    buf_cell.set_style(Style::reset().fg(fg).bg(bg).add_modifier(modifier));
}

#[inline]
fn map_color(color: vt100::Color) -> ratatui_core::style::Color {
    match color {
        vt100::Color::Default => ratatui_core::style::Color::Reset,
        vt100::Color::Idx(i) => ratatui_core::style::Color::Indexed(i),
        vt100::Color::Rgb(r, g, b) => ratatui_core::style::Color::Rgb(r, g, b),
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
