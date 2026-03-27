use std::borrow::Cow;

use ratatui_core::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};
use ratatui_widgets::{block::Block, clear::Clear};

use crate::state;

/// A trait representing a pseudo-terminal screen.
///
/// Implementing this trait allows for backends other than `vt100` to be used
/// with the `PseudoTerminal` widget.
///
/// All row coordinates use a visible coordinate system where row 0 is the
/// topmost row currently displayed. When scrollback is active, this may
/// differ from the underlying terminal buffer's row numbering. Implementors
/// must ensure that [`cell`](Self::cell) and [`cursor_position`](Self::cursor_position)
/// use the same coordinate space.
pub trait Screen {
    /// The type of cell this screen contains
    type C: Cell;

    /// Returns the cell at the given location in visible coordinates.
    ///
    /// Row 0 is the topmost visible row, accounting for any scrollback offset.
    fn cell(&self, row: u16, col: u16) -> Option<&Self::C>;
    /// Returns whether the cursor should be hidden.
    fn hide_cursor(&self) -> bool;
    /// Returns the cursor position in visible coordinates.
    ///
    /// The return value is (row, column), where row 0 is the topmost visible
    /// row. This must use the same coordinate space as [`cell`](Self::cell).
    /// When scrollback is active, the cursor row must be shifted down by the
    /// number of scrollback rows visible above the active screen content.
    ///
    /// If the cursor is not within the visible area (e.g., scrolled
    /// off-screen), the returned row may exceed the screen dimensions.
    /// The rendering layer handles bounds checking.
    fn cursor_position(&self) -> (u16, u16);
}

/// A trait for representing a single cell on a screen.
pub trait Cell {
    /// Whether the cell has any contents that could be rendered to the screen.
    fn has_contents(&self) -> bool;
    /// Apply the contents and styling of this cell to the provided buffer cell.
    fn apply(&self, cell: &mut ratatui_core::buffer::Cell);
}

/// A widget representing a pseudo-terminal screen.
///
/// The `PseudoTerminal` widget displays the contents of a pseudo-terminal screen,
/// which is typically populated with text and control sequences from a terminal emulator.
/// It provides a visual representation of the terminal output within a TUI application.
///
/// The contents of the pseudo-terminal screen are represented by a `vt100::Screen` object.
/// The `vt100` library provides functionality for parsing and processing terminal control sequences
/// and handling terminal state, allowing the `PseudoTerminal` widget to accurately render the
/// terminal output.
///
/// # Examples
///
/// ```rust
/// use ratatui_core::style::{Color, Modifier, Style};
/// use ratatui_widgets::{block::Block, borders::Borders};
/// use tui_term::widget::PseudoTerminal;
/// use vt100::Parser;
///
/// let mut parser = vt100::Parser::new(24, 80, 0);
/// let pseudo_term = PseudoTerminal::new(parser.screen())
///     .block(Block::default().title("Terminal").borders(Borders::ALL))
///     .style(
///         Style::default()
///             .fg(Color::White)
///             .bg(Color::Black)
///             .add_modifier(Modifier::BOLD),
///     );
/// ```
#[non_exhaustive]
pub struct PseudoTerminal<'a, S> {
    screen: &'a S,
    pub(crate) block: Option<Block<'a>>,
    style: Option<Style>,
    pub(crate) cursor: Cursor,
}

#[non_exhaustive]
pub struct Cursor {
    pub(crate) show: bool,
    pub(crate) symbol: Cow<'static, str>,
    pub(crate) style: Style,
    pub(crate) overlay_style: Style,
}

impl Cursor {
    /// Sets the symbol for the cursor.
    ///
    /// # Arguments
    ///
    /// * `symbol`: The symbol to set as the cursor.
    ///
    /// # Example
    ///
    /// ```
    /// use ratatui_core::style::Style;
    /// use tui_term::widget::Cursor;
    ///
    /// let cursor = Cursor::default().symbol("|");
    /// ```
    #[inline]
    #[must_use]
    pub fn symbol(mut self, symbol: impl Into<Cow<'static, str>>) -> Self {
        self.symbol = symbol.into();
        self
    }

    /// Sets the style for the cursor.
    ///
    /// # Arguments
    ///
    /// * `style`: The `Style` to set for the cursor.
    ///
    /// # Example
    ///
    /// ```
    /// use ratatui_core::style::Style;
    /// use tui_term::widget::Cursor;
    ///
    /// let cursor = Cursor::default().style(Style::default());
    /// ```
    #[inline]
    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the overlay style for the cursor.
    ///
    /// The overlay style is used when the cursor overlaps with existing content on the screen.
    ///
    /// # Arguments
    ///
    /// * `overlay_style`: The `Style` to set as the overlay style for the cursor.
    ///
    /// # Example
    ///
    /// ```
    /// use ratatui_core::style::Style;
    /// use tui_term::widget::Cursor;
    ///
    /// let cursor = Cursor::default().overlay_style(Style::default());
    /// ```
    #[inline]
    #[must_use]
    pub const fn overlay_style(mut self, overlay_style: Style) -> Self {
        self.overlay_style = overlay_style;
        self
    }

    /// Set the visibility of the cursor (default = shown)
    #[inline]
    #[must_use]
    pub const fn visibility(mut self, show: bool) -> Self {
        self.show = show;
        self
    }

    /// Show the cursor (default)
    #[inline]
    pub fn show(&mut self) {
        self.show = true;
    }

    /// Hide the cursor
    #[inline]
    pub fn hide(&mut self) {
        self.show = false;
    }
}

impl Default for Cursor {
    #[inline]
    fn default() -> Self {
        Self {
            show: true,
            symbol: Cow::Borrowed("\u{2588}"), //"█".
            style: Style::default().fg(Color::Gray),
            overlay_style: Style::default().add_modifier(Modifier::REVERSED),
        }
    }
}

impl<'a, S: Screen> PseudoTerminal<'a, S> {
    /// Creates a new instance of `PseudoTerminal`.
    ///
    /// # Arguments
    ///
    /// * `screen`: The reference to the `Screen`.
    ///
    /// # Example
    ///
    /// ```
    /// use tui_term::widget::PseudoTerminal;
    /// use vt100::Parser;
    ///
    /// let mut parser = vt100::Parser::new(24, 80, 0);
    /// let pseudo_term = PseudoTerminal::new(parser.screen());
    /// ```
    #[inline]
    #[must_use]
    pub fn new(screen: &'a S) -> Self {
        PseudoTerminal {
            screen,
            block: None,
            style: None,
            cursor: Cursor::default(),
        }
    }

    /// Sets the block for the `PseudoTerminal`.
    ///
    /// # Arguments
    ///
    /// * `block`: The `Block` to set.
    ///
    /// # Example
    ///
    /// ```
    /// use ratatui_widgets::block::Block;
    /// use tui_term::widget::PseudoTerminal;
    /// use vt100::Parser;
    ///
    /// let mut parser = vt100::Parser::new(24, 80, 0);
    /// let block = Block::default();
    /// let pseudo_term = PseudoTerminal::new(parser.screen()).block(block);
    /// ```
    #[inline]
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Sets the cursor configuration for the `PseudoTerminal`.
    ///
    /// The `cursor` method allows configuring the appearance of the cursor within the
    /// `PseudoTerminal` widget.
    ///
    /// # Arguments
    ///
    /// * `cursor`: The `Cursor` configuration to set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ratatui_core::style::Style;
    /// use tui_term::widget::{Cursor, PseudoTerminal};
    ///
    /// let mut parser = vt100::Parser::new(24, 80, 0);
    /// let cursor = Cursor::default().symbol("|").style(Style::default());
    /// let pseudo_term = PseudoTerminal::new(parser.screen()).cursor(cursor);
    /// ```
    #[inline]
    #[must_use]
    pub fn cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = cursor;
        self
    }

    /// Sets the style for `PseudoTerminal`.
    ///
    /// # Arguments
    ///
    /// * `style`: The `Style` to set.
    ///
    /// # Example
    ///
    /// ```
    /// use ratatui_core::style::Style;
    /// use tui_term::widget::PseudoTerminal;
    ///
    /// let mut parser = vt100::Parser::new(24, 80, 0);
    /// let style = Style::default();
    /// let pseudo_term = PseudoTerminal::new(parser.screen()).style(style);
    /// ```
    #[inline]
    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    #[inline]
    #[must_use]
    pub const fn screen(&self) -> &S {
        self.screen
    }
}

impl<S: Screen> Widget for &PseudoTerminal<'_, S> {
    #[inline]
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let area = self.block.as_ref().map_or(area, |b| {
            let inner_area = b.inner(area);
            b.clone().render(area, buf);
            inner_area
        });
        state::handle(self, area, buf);
    }
}

impl<S: Screen> Widget for PseudoTerminal<'_, S> {
    #[inline]
    fn render(self, area: Rect, buf: &mut Buffer) {
        (&self).render(area, buf);
    }
}

#[cfg(all(test, feature = "vt100"))]
mod tests {
    use ratatui::Terminal;
    use ratatui_core::backend::TestBackend;
    use ratatui_widgets::borders::Borders;

    use super::*;

    fn snapshot_typescript(stream: &[u8]) -> String {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(24, 80, 0);
        parser.process(stream);
        let pseudo_term = PseudoTerminal::new(parser.screen());
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.area());
            })
            .unwrap();
        format!("{:?}", terminal.backend().buffer())
    }

    #[test]
    fn scrollback_cursor_not_rendered_when_off_screen() {
        let backend = TestBackend::new(40, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(4, 40, 20);

        // 10 lines on 4 rows: 6 scroll off. With set_scrollback(4) all
        // visible rows are scrollback. Cursor maps to visible row 7, off-screen.
        for i in 0..10 {
            parser.process(format!("[scrollback line {i}]\r\n").as_bytes());
        }
        parser.screen_mut().set_scrollback(4);

        let pseudo_term = PseudoTerminal::new(parser.screen());
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.area());
            })
            .unwrap();

        let buf = terminal.backend().buffer();
        for row in 0..4 {
            for col in 0..40 {
                let cell = &buf[(col, row)];
                assert!(
                    !cell.modifier.contains(Modifier::REVERSED),
                    "REVERSED cursor style should not appear at ({col}, {row}) \
                     when cursor is off-screen due to scrollback"
                );
            }
        }
        let view = format!("{:?}", terminal.backend().buffer());
        insta::assert_snapshot!(view);
    }

    #[test]
    fn scrollback_partial_cursor_visible_at_adjusted_row() {
        let backend = TestBackend::new(40, 6);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(6, 40, 20);

        // 8 lines on 6 rows: 3 scroll off into scrollback.
        // \x1b[H moves cursor to drawing row 0.
        parser.process(b"[scrollback: hidden 1]\r\n");
        parser.process(b"[scrollback: hidden 2]\r\n");
        parser.process(b"[scrollback: shown at row 0]\r\n");
        parser.process(b"[draw-0: cursor row]\r\n");
        parser.process(b"[draw-1]\r\n");
        parser.process(b"[draw-2]\r\n");
        parser.process(b"[draw-3]\r\n");
        parser.process(b"[draw-4]\r\n");
        parser.process(b"\x1b[H");

        // With scrollback=1, drawing row 0 shifts to visible row 1.
        // Visible row 0 becomes the last scrollback line.
        parser.screen_mut().set_scrollback(1);

        let pseudo_term = PseudoTerminal::new(parser.screen());
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.area());
            })
            .unwrap();

        let buf = terminal.backend().buffer();
        for col in 0..40 {
            let cell = &buf[(col, 0)];
            assert!(
                !cell.modifier.contains(Modifier::REVERSED),
                "scrollback row 0 at col {col} should not have cursor styling"
            );
        }
        let cursor_cell = &buf[(0, 1)];
        assert!(
            cursor_cell.modifier.contains(Modifier::REVERSED),
            "cursor should render at visible row 1 (adjusted from drawing row 0)"
        );
        let view = format!("{:?}", terminal.backend().buffer());
        insta::assert_snapshot!(view);
    }

    #[test]
    fn scrollback_hide_cursor_suppresses_rendering() {
        let backend = TestBackend::new(40, 6);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(6, 40, 20);

        for i in 0..8 {
            parser.process(format!("line {i}\r\n").as_bytes());
        }
        parser.process(b"\x1b[H");
        // Hide cursor via escape sequence
        parser.process(b"\x1b[?25l");
        parser.screen_mut().set_scrollback(1);

        let pseudo_term = PseudoTerminal::new(parser.screen());
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.area());
            })
            .unwrap();

        let buf = terminal.backend().buffer();
        for row in 0..6 {
            for col in 0..40 {
                let cell = &buf[(col, row)];
                assert!(
                    !cell.modifier.contains(Modifier::REVERSED),
                    "hidden cursor should not style ({col}, {row}) even with scrollback"
                );
            }
        }
    }

    #[test]
    fn scrollback_cursor_with_block() {
        let backend = TestBackend::new(42, 8);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(6, 40, 20);

        for i in 0..8 {
            parser.process(format!("line {i}\r\n").as_bytes());
        }
        parser.process(b"\x1b[H");
        parser.screen_mut().set_scrollback(1);

        let block = Block::default().borders(Borders::ALL).title("pty");
        let pseudo_term = PseudoTerminal::new(parser.screen()).block(block);
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.area());
            })
            .unwrap();

        // Border takes 1 row/col on each side, so inner area starts at (1,1).
        // Visible row 0 is scrollback (inner buf row 1), no cursor.
        // Cursor at drawing row 0 + scrollback 1 = visible row 1 (inner buf row 2).
        let buf = terminal.backend().buffer();
        let scrollback_cell = &buf[(1, 1)];
        assert!(
            !scrollback_cell.modifier.contains(Modifier::REVERSED),
            "scrollback row inside block should not have cursor styling"
        );
        let cursor_cell = &buf[(1, 2)];
        assert!(
            cursor_cell.modifier.contains(Modifier::REVERSED),
            "cursor should render at inner row 1 (buf row 2) with block border"
        );
    }

    #[test]
    fn empty_actions() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(24, 80, 0);
        parser.process(b" ");
        let pseudo_term = PseudoTerminal::new(parser.screen());
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.area());
            })
            .unwrap();
        let view = format!("{:?}", terminal.backend().buffer());
        insta::assert_snapshot!(view);
    }
    #[test]
    fn boundary_rows_overshot_no_panic() {
        let stream = include_bytes!("../test/typescript/simple_ls.typescript");
        // Make the backend on purpose much smaller
        let backend = TestBackend::new(80, 4);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(24, 80, 0);
        parser.process(stream);
        let pseudo_term = PseudoTerminal::new(parser.screen());
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.area());
            })
            .unwrap();
        let view = format!("{:?}", terminal.backend().buffer());
        insta::assert_snapshot!(view);
    }

    #[test]
    fn simple_ls() {
        let stream = include_bytes!("../test/typescript/simple_ls.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn simple_cursor_alternate_symbol() {
        let stream = include_bytes!("../test/typescript/simple_ls.typescript");
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(24, 80, 0);
        let cursor = Cursor::default().symbol("|");
        parser.process(stream);
        let pseudo_term = PseudoTerminal::new(parser.screen()).cursor(cursor);
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.area());
            })
            .unwrap();
        let view = format!("{:?}", terminal.backend().buffer());
        insta::assert_snapshot!(view);
    }
    #[test]
    fn simple_cursor_styled() {
        let stream = include_bytes!("../test/typescript/simple_ls.typescript");
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(24, 80, 0);
        let style = Style::default().bg(Color::Cyan).fg(Color::LightRed);
        let cursor = Cursor::default().symbol("|").style(style);
        parser.process(stream);
        let pseudo_term = PseudoTerminal::new(parser.screen()).cursor(cursor);
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.area());
            })
            .unwrap();
        let view = format!("{:?}", terminal.backend().buffer());
        insta::assert_snapshot!(view);
    }
    #[test]
    fn simple_cursor_hide() {
        let stream = include_bytes!("../test/typescript/simple_ls.typescript");
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(24, 80, 0);
        let cursor = Cursor::default().visibility(false);
        parser.process(stream);
        let pseudo_term = PseudoTerminal::new(parser.screen()).cursor(cursor);
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.area());
            })
            .unwrap();
        let view = format!("{:?}", terminal.backend().buffer());
        insta::assert_snapshot!(view);
    }
    #[test]
    fn simple_cursor_hide_alt() {
        let stream = include_bytes!("../test/typescript/simple_ls.typescript");
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(24, 80, 0);
        let mut cursor = Cursor::default();
        cursor.hide();
        parser.process(stream);
        let pseudo_term = PseudoTerminal::new(parser.screen()).cursor(cursor);
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.area());
            })
            .unwrap();
        let view = format!("{:?}", terminal.backend().buffer());
        insta::assert_snapshot!(view);
    }
    #[test]
    fn overlapping_cursor() {
        let stream = include_bytes!("../test/typescript/overlapping_cursor.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn overlapping_cursor_alternate_style() {
        let stream = include_bytes!("../test/typescript/overlapping_cursor.typescript");
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(24, 80, 0);
        let style = Style::default().bg(Color::Cyan).fg(Color::LightRed);
        let cursor = Cursor::default().overlay_style(style);
        parser.process(stream);
        let pseudo_term = PseudoTerminal::new(parser.screen()).cursor(cursor);
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.area());
            })
            .unwrap();
        let view = format!("{:?}", terminal.backend().buffer());
        insta::assert_snapshot!(view);
    }
    #[test]
    fn simple_ls_with_block() {
        let stream = include_bytes!("../test/typescript/simple_ls.typescript");
        let backend = TestBackend::new(100, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(24, 80, 0);
        parser.process(stream);
        let block = Block::default().borders(Borders::ALL).title("ls");
        let pseudo_term = PseudoTerminal::new(parser.screen()).block(block);
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.area());
            })
            .unwrap();
        let view = format!("{:?}", terminal.backend().buffer());
        insta::assert_snapshot!(view);
    }
    #[test]
    fn simple_ls_no_style_from_block() {
        let stream = include_bytes!("../test/typescript/simple_ls.typescript");
        let backend = TestBackend::new(100, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(24, 80, 0);
        parser.process(stream);
        let block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().add_modifier(Modifier::BOLD))
            .title("ls");
        let pseudo_term = PseudoTerminal::new(parser.screen()).block(block);
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.area());
            })
            .unwrap();
        let view = format!("{:?}", terminal.backend().buffer());
        insta::assert_snapshot!(view);
    }
    #[test]
    fn italic_text() {
        let stream = b"[3mThis line will be displayed in italic.[0m This should have no style.";
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn underlined_text() {
        let stream =
            b"[4mThis line will be displayed with an underline.[0m This should have no style.";
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn bold_text() {
        let stream = b"[1mThis line will be displayed bold.[0m This should have no style.";
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn inverse_text() {
        let stream = b"[7mThis line will be displayed inversed.[0m This should have no style.";
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn dim_text() {
        let stream =
            b"\x1b[2mThis line will be displayed dim/faint.\x1b[0m This should have no style.";
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn combined_modifier_text() {
        let stream =
            b"[4m[3mThis line will be displayed in italic and underlined.[0m This should have no style.";
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn dim_bold_text() {
        let stream = b"\x1b[2m\x1b[1mThis is dim and bold. Bold takes precedence.\x1b[0m Normal.";
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_02_01() {
        let stream = include_bytes!("../test/typescript/vttest_02_01.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_02() {
        let stream = include_bytes!("../test/typescript/vttest_02_02.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_03() {
        let stream = include_bytes!("../test/typescript/vttest_02_03.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_04() {
        let stream = include_bytes!("../test/typescript/vttest_02_04.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_05() {
        let stream = include_bytes!("../test/typescript/vttest_02_05.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_06() {
        let stream = include_bytes!("../test/typescript/vttest_02_06.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_07() {
        let stream = include_bytes!("../test/typescript/vttest_02_07.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_08() {
        let stream = include_bytes!("../test/typescript/vttest_02_08.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_09() {
        let stream = include_bytes!("../test/typescript/vttest_02_09.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_10() {
        let stream = include_bytes!("../test/typescript/vttest_02_10.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_11() {
        let stream = include_bytes!("../test/typescript/vttest_02_11.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_12() {
        let stream = include_bytes!("../test/typescript/vttest_02_12.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_13() {
        let stream = include_bytes!("../test/typescript/vttest_02_13.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_14() {
        let stream = include_bytes!("../test/typescript/vttest_02_14.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_15() {
        let stream = include_bytes!("../test/typescript/vttest_02_15.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_03_01() {
        let stream = include_bytes!("../test/typescript/vttest_03_01.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_01_01() {
        let stream = include_bytes!("../test/typescript/vttest_01_01.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_01_02() {
        let stream = include_bytes!("../test/typescript/vttest_01_02.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_01_03() {
        let stream = include_bytes!("../test/typescript/vttest_01_03.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_01_04() {
        let stream = include_bytes!("../test/typescript/vttest_01_04.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_01_05() {
        let stream = include_bytes!("../test/typescript/vttest_01_05.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_menu() {
        let stream = include_bytes!("../test/typescript/vttest_menu.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_08_01() {
        let stream = include_bytes!("../test/typescript/vttest_08_01.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_08_02() {
        let stream = include_bytes!("../test/typescript/vttest_08_02.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_08_03() {
        let stream = include_bytes!("../test/typescript/vttest_08_03.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_08_04() {
        let stream = include_bytes!("../test/typescript/vttest_08_04.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_08_05() {
        let stream = include_bytes!("../test/typescript/vttest_08_05.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_08_06() {
        let stream = include_bytes!("../test/typescript/vttest_08_06.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_08_07() {
        let stream = include_bytes!("../test/typescript/vttest_08_07.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_08_08() {
        let stream = include_bytes!("../test/typescript/vttest_08_08.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_08_09() {
        let stream = include_bytes!("../test/typescript/vttest_08_09.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_08_10() {
        let stream = include_bytes!("../test/typescript/vttest_08_10.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_08_11() {
        let stream = include_bytes!("../test/typescript/vttest_08_11.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_08_12() {
        let stream = include_bytes!("../test/typescript/vttest_08_12.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_08_13() {
        let stream = include_bytes!("../test/typescript/vttest_08_13.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_08_14() {
        let stream = include_bytes!("../test/typescript/vttest_08_14.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_11_06_02() {
        let stream = include_bytes!("../test/typescript/vttest_11_06_02.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_11_06_03() {
        let stream = include_bytes!("../test/typescript/vttest_11_06_03.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_11_06_04() {
        let stream = include_bytes!("../test/typescript/vttest_11_06_04.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_11_06_05() {
        let stream = include_bytes!("../test/typescript/vttest_11_06_05.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_09_01() {
        let stream = include_bytes!("../test/typescript/vttest_09_01.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_09_02() {
        let stream = include_bytes!("../test/typescript/vttest_09_02.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_09_03() {
        let stream = include_bytes!("../test/typescript/vttest_09_03.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_09_04() {
        let stream = include_bytes!("../test/typescript/vttest_09_04.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_09_05() {
        let stream = include_bytes!("../test/typescript/vttest_09_05.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_09_06() {
        let stream = include_bytes!("../test/typescript/vttest_09_06.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_09_07() {
        let stream = include_bytes!("../test/typescript/vttest_09_07.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_09_08() {
        let stream = include_bytes!("../test/typescript/vttest_09_08.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_09_09() {
        let stream = include_bytes!("../test/typescript/vttest_09_09.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_11_05_01() {
        let stream = include_bytes!("../test/typescript/vttest_11_05_01.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_11_05_02() {
        let stream = include_bytes!("../test/typescript/vttest_11_05_02.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_11_05_03() {
        let stream = include_bytes!("../test/typescript/vttest_11_05_03.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_11_05_04() {
        let stream = include_bytes!("../test/typescript/vttest_11_05_04.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_11_05_05() {
        let stream = include_bytes!("../test/typescript/vttest_11_05_05.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_11_05_06() {
        let stream = include_bytes!("../test/typescript/vttest_11_05_06.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_11_05_07() {
        let stream = include_bytes!("../test/typescript/vttest_11_05_07.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_11_05_08() {
        let stream = include_bytes!("../test/typescript/vttest_11_05_08.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_11_05_09() {
        let stream = include_bytes!("../test/typescript/vttest_11_05_09.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }

    #[test]
    fn vttest_11_07_02() {
        let stream = include_bytes!("../test/typescript/vttest_11_07_02.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
}
