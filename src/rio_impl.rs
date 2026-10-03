use ratatui_core::style::{Color, Modifier, Style as RStyle};
use rio_vt::config::colors::{AnsiColor, NamedColor};
use rio_vt::crosswords::pos::Column;
use rio_vt::crosswords::square::ContentTag;
use rio_vt::crosswords::style::{Style, StyleFlags};
use rio_vt::crosswords::{Crosswords, Mode};
use rio_vt::event::EventListener;

use crate::widget::{Cell, Screen};

/// A visible cell resolved from a rio-vt `Square` into ratatui form.
pub struct RioCell {
    symbol: char,
    style: RStyle,
    has_contents: bool,
}

impl Cell for RioCell {
    #[inline]
    fn has_contents(&self) -> bool {
        self.has_contents
    }

    #[inline]
    fn apply(&self, buf_cell: &mut ratatui_core::buffer::Cell) {
        if self.has_contents {
            let mut buf = [0u8; 4];
            buf_cell.set_symbol(self.symbol.encode_utf8(&mut buf));
        }
        buf_cell.set_style(self.style);
    }
}

/// A snapshot of a rio-vt terminal's visible screen.
///
/// rio-vt cells are packed handles into the grid style table, so we resolve the
/// visible screen up front rather than borrowing it like the `vt100` backend.
pub struct RioScreen {
    columns: u16,
    cells: Vec<RioCell>,
    cursor: (u16, u16),
    hide_cursor: bool,
}

impl RioScreen {
    /// Materialize the visible screen of a rio-vt terminal.
    pub fn new<L: EventListener>(term: &Crosswords<L>) -> Self {
        let columns = term.columns();
        let rows = term.visible_rows();
        let styles = term.grid.style_set.styles();

        let mut cells = Vec::with_capacity(rows.len() * columns);
        for row in &rows {
            for col in 0..columns {
                let square = row[Column(col)];
                let symbol = square.c();
                let (fg, bg, flags) = match square.content_tag() {
                    ContentTag::Codepoint => {
                        let style: Style = styles
                            .get(square.style_id() as usize)
                            .copied()
                            .unwrap_or_default();
                        (map_color(style.fg), map_color(style.bg), style.flags)
                    }
                    ContentTag::BgPalette => (
                        Color::Reset,
                        Color::Indexed(square.bg_palette_index()),
                        StyleFlags::empty(),
                    ),
                    ContentTag::BgRgb => {
                        let (r, g, b) = square.bg_rgb();
                        (Color::Reset, Color::Rgb(r, g, b), StyleFlags::empty())
                    }
                };
                cells.push(RioCell {
                    symbol,
                    style: build_style(fg, bg, flags),
                    has_contents: symbol != ' ' && symbol != '\u{0}',
                });
            }
        }

        let cursor = term.cursor();
        Self {
            columns: columns as u16,
            cells,
            cursor: (
                u16::try_from(cursor.pos.row.0.max(0)).unwrap_or(u16::MAX),
                u16::try_from(cursor.pos.col.0).unwrap_or(u16::MAX),
            ),
            hide_cursor: !term.mode().contains(Mode::SHOW_CURSOR),
        }
    }
}

impl Screen for RioScreen {
    type C = RioCell;

    #[inline]
    fn cell(&self, row: u16, col: u16) -> Option<&Self::C> {
        if col >= self.columns {
            return None;
        }
        let idx = row as usize * self.columns as usize + col as usize;
        self.cells.get(idx)
    }

    #[inline]
    fn hide_cursor(&self) -> bool {
        self.hide_cursor
    }

    #[inline]
    fn cursor_position(&self) -> (u16, u16) {
        self.cursor
    }
}

#[inline]
fn build_style(fg: Color, bg: Color, flags: StyleFlags) -> RStyle {
    let mut modifier = Modifier::empty();
    if flags.contains(StyleFlags::BOLD) {
        modifier |= Modifier::BOLD;
    }
    if flags.contains(StyleFlags::ITALIC) {
        modifier |= Modifier::ITALIC;
    }
    if flags.intersects(StyleFlags::ALL_UNDERLINES) {
        modifier |= Modifier::UNDERLINED;
    }
    if flags.contains(StyleFlags::INVERSE) {
        modifier |= Modifier::REVERSED;
    }
    if flags.contains(StyleFlags::DIM) {
        modifier |= Modifier::DIM;
    }
    if flags.contains(StyleFlags::STRIKEOUT) {
        modifier |= Modifier::CROSSED_OUT;
    }
    if flags.contains(StyleFlags::HIDDEN) {
        modifier |= Modifier::HIDDEN;
    }
    RStyle::reset().fg(fg).bg(bg).add_modifier(modifier)
}

#[inline]
fn map_color(color: AnsiColor) -> Color {
    match color {
        AnsiColor::Named(NamedColor::Foreground | NamedColor::Background) => Color::Reset,
        AnsiColor::Named(named) => {
            let index = named as u32;
            if index < 16 {
                Color::Indexed(index as u8)
            } else {
                Color::Reset
            }
        }
        AnsiColor::Indexed(index) => Color::Indexed(index),
        AnsiColor::Spec(rgb) => Color::Rgb(rgb.r, rgb.g, rgb.b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::PseudoTerminal;
    use ratatui::Terminal;
    use ratatui_core::backend::TestBackend;
    use rio_vt::ansi::CursorShape;
    use rio_vt::crosswords::{Crosswords, CrosswordsSize};
    use rio_vt::event::{VoidListener, WindowId};
    use rio_vt::performer::handler::Processor;

    fn crosswords(cols: usize, rows: usize, bytes: &[u8]) -> Crosswords<VoidListener> {
        let mut term = Crosswords::new(
            CrosswordsSize::new(cols, rows),
            CursorShape::Block,
            VoidListener,
            WindowId::from(0),
            0,
            0,
        );
        Processor::default().advance(&mut term, bytes);
        term
    }

    #[test]
    fn renders_text_and_sgr_color() {
        // "hi", then a red "R", then reset.
        let term = crosswords(80, 24, b"hi\x1b[31mR\x1b[0m");
        let screen = RioScreen::new(&term);

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| f.render_widget(PseudoTerminal::new(&screen), f.area()))
            .unwrap();

        let buf = terminal.backend().buffer();
        assert_eq!(buf[(0, 0)].symbol(), "h");
        assert_eq!(buf[(1, 0)].symbol(), "i");
        assert_eq!(buf[(2, 0)].symbol(), "R");
        assert_eq!(buf[(2, 0)].fg, Color::Indexed(1), "SGR 31 is red");
    }

    #[test]
    fn cursor_position_and_visibility() {
        let term = crosswords(80, 24, b"abc");
        let screen = RioScreen::new(&term);
        assert_eq!(screen.cursor_position(), (0, 3));
        assert!(!screen.hide_cursor());

        // DECTCEM hide.
        let term = crosswords(80, 24, b"abc\x1b[?25l");
        let screen = RioScreen::new(&term);
        assert!(screen.hide_cursor());
    }
}
