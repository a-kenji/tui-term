use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Clear, Widget};
use vt100::Screen;

use crate::state;

/// A widget representing a pseudo-terminal screen.
///
/// The `PseudoTerminal` widget displays the contents of a pseudo-terminal screen,
/// which is typically populated with text and control sequences from a terminal emulator.
/// It provides a visual representation of the terminal output within a TUI application.
///
/// The contents of the pseudo-terminal screen are represented by a `vt100::Screen` object.
/// The `vt100` library provides functionality for parsing and processing terminal control sequences and handling terminal state,
/// allowing the `PseudoTerminal` widget to accurately render the terminal output.
///
/// # Examples
///
/// ```rust
/// use ratatui::widgets::{Block, Borders};
/// use ratatui::style::{Style, Modifier, Color};
/// use tui_term::widget::PseudoTerminal;
/// use vt100::Parser;
///
/// let mut parser = vt100::Parser::new(24, 80, 0);
/// let pseudo_term = PseudoTerminal::new(&parser.screen())
///     .block(Block::default().title("Terminal").borders(Borders::ALL))
///     .style(Style::default().fg(Color::White).bg(Color::Black).add_modifier(Modifier::BOLD));
/// ```
pub struct PseudoTerminal<'a> {
    screen: &'a Screen,
    pub(crate) block: Option<Block<'a>>,
    style: Option<Style>,
    pub(crate) cursor: Cursor,
}

pub struct Cursor {
    pub(crate) symbol: String,
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
    /// use tui_term::widget::Cursor;
    /// use ratatui::style::Style;
    ///
    /// let cursor = Cursor::default().symbol("|");
    /// ```
    #[inline]
    #[must_use]
    pub fn symbol(mut self, symbol: &str) -> Self {
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
    /// use tui_term::widget::Cursor;
    /// use ratatui::style::Style;
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
    /// use tui_term::widget::Cursor;
    /// use ratatui::style::Style;
    ///
    /// let cursor = Cursor::default().overlay_style(Style::default());
    /// ```
    #[inline]
    #[must_use]
    pub const fn overlay_style(mut self, overlay_style: Style) -> Self {
        self.overlay_style = overlay_style;
        self
    }
}

impl Default for Cursor {
    #[inline]
    fn default() -> Self {
        Self {
            symbol: "\u{2588}".into(), //"█".
            style: Style::default().fg(Color::Gray),
            overlay_style: Style::default().add_modifier(Modifier::REVERSED),
        }
    }
}

impl<'a> PseudoTerminal<'a> {
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
    /// let pseudo_term = PseudoTerminal::new(&parser.screen());
    /// ```
    #[inline]
    #[must_use]
    pub fn new(screen: &'a Screen) -> Self {
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
    /// use tui_term::widget::PseudoTerminal;
    /// use ratatui::widgets::Block;
    /// use vt100::Parser;
    ///
    /// let mut parser = vt100::Parser::new(24, 80, 0);
    /// let block = Block::default();
    /// let pseudo_term = PseudoTerminal::new(&parser.screen()).block(block);
    /// ```
    #[inline]
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    /// Sets the cursor configuration for the `PseudoTerminal`.
    ///
    /// The `cursor` method allows configuring the appearance of the cursor within the `PseudoTerminal` widget.
    ///
    /// # Arguments
    ///
    /// * `cursor`: The `Cursor` configuration to set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tui_term::widget::PseudoTerminal;
    /// use tui_term::widget::Cursor;
    /// use ratatui::style::Style;
    ///
    /// let mut parser = vt100::Parser::new(24, 80, 0);
    /// let cursor = Cursor::default().symbol("|").style(Style::default());
    /// let pseudo_term = PseudoTerminal::new(&parser.screen()).cursor(cursor);
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
    /// use tui_term::widget::PseudoTerminal;
    /// use ratatui::style::Style;
    ///
    /// let mut parser = vt100::Parser::new(24, 80, 0);
    /// let style = Style::default();
    /// let pseudo_term = PseudoTerminal::new(&parser.screen()).style(style);
    /// ```
    #[inline]
    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    #[inline]
    #[must_use]
    pub const fn screen(&self) -> &Screen {
        self.screen
    }
}

impl Widget for PseudoTerminal<'_> {
    #[inline]
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let area = self.block.as_ref().map_or(area, |b| {
            let inner_area = b.inner(area);
            b.clone().render(area, buf);
            inner_area
        });
        state::handle(&self, area, buf);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::widgets::Borders;
    use ratatui::Terminal;

    use super::*;

    fn snapshot_typescript(stream: &[u8]) -> String {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(24, 80, 0);
        parser.process(stream);
        let pseudo_term = PseudoTerminal::new(parser.screen());
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.size());
            })
            .unwrap();
        format!("{:?}", terminal.backend().buffer())
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
                f.render_widget(pseudo_term, f.size());
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
                f.render_widget(pseudo_term, f.size());
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
                f.render_widget(pseudo_term, f.size());
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
                f.render_widget(pseudo_term, f.size());
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
                f.render_widget(pseudo_term, f.size());
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
                f.render_widget(pseudo_term, f.size());
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
                f.render_widget(pseudo_term, f.size());
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
    fn combined_modifier_text() {
        let stream =
            b"[4m[3mThis line will be displayed in italic and underlined.[0m This should have no style.";
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
}
