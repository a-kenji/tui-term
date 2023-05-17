use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Clear, Widget};
use vt100::Screen;

use crate::state;

/// A widget representing a pseudo-terminal screen.
///
/// The `PseudoTerm` widget displays the contents of a pseudo-terminal screen,
/// which is typically populated with text and control sequences from a terminal emulator.
/// It provides a visual representation of the terminal output within a TUI application.
///
/// The contents of the pseudo-terminal screen are represented by a `vt100::Screen` object.
/// The `vt100` library provides functionality for parsing and processing terminal control sequences and handling terminal state,
/// allowing the `PseudoTerm` widget to accurately render the terminal output.
///
/// # Examples
///
/// ```rust
/// use ratatui::widgets::{Block, Borders};
/// use ratatui::style::{Style, Modifier, Color};
/// use tui_term::widget::PseudoTerm;
/// use vt100::Parser;
///
/// let mut parser = vt100::Parser::new(24, 80, 0);
/// let pseudo_term = PseudoTerm::new(&parser.screen())
///     .block(Block::default().title("Terminal").borders(Borders::ALL))
///     .style(Style::default().fg(Color::White).bg(Color::Black).add_modifier(Modifier::BOLD));
/// ```
pub struct PseudoTerm<'a> {
    screen: &'a Screen,
    pub(crate) block: Option<Block<'a>>,
    style: Option<Style>,
}

impl<'a> PseudoTerm<'a> {
    /// Creates a new instance of `PseudoTerm`.
    ///
    /// # Arguments
    ///
    /// * `screen`: The reference to the `Screen`.
    ///
    /// # Example
    ///
    /// ```
    /// use tui_term::widget::PseudoTerm;
    /// use vt100::Parser;
    ///
    /// let mut parser = vt100::Parser::new(24, 80, 0);
    /// let pseudo_term = PseudoTerm::new(&parser.screen());
    /// ```
    pub fn new(screen: &'a Screen) -> Self {
        PseudoTerm {
            screen,
            block: None,
            style: None,
        }
    }
    /// Sets the block for the `PseudoTerm`.
    ///
    /// # Arguments
    ///
    /// * `block`: The `Block` to set.
    ///
    /// # Example
    ///
    /// ```
    /// use tui_term::widget::PseudoTerm;
    /// use ratatui::widgets::Block;
    /// use vt100::Parser;
    ///
    /// let mut parser = vt100::Parser::new(24, 80, 0);
    /// let block = Block::default();
    /// let pseudo_term = PseudoTerm::new(&parser.screen()).block(block);
    /// ```
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
    /// Sets the style for `PseudoTerm`.
    ///
    /// # Arguments
    ///
    /// * `style`: The `Style` to set.
    ///
    /// # Example
    ///
    /// ```
    /// use tui_term::widget::PseudoTerm;
    /// use ratatui::style::Style;
    ///
    /// let mut parser = vt100::Parser::new(24, 80, 0);
    /// let style = Style::default();
    /// let pseudo_term = PseudoTerm::new(&parser.screen()).style(style);
    /// ```
    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    pub fn screen(&self) -> &Screen {
        self.screen
    }
}

impl Widget for PseudoTerm<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let area = match &self.block {
            Some(b) => {
                let inner_area = b.inner(area);
                b.clone().render(area, buf);
                inner_area
            }
            None => area,
        };
        state::handle(&self, &area, buf);
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
        let pseudo_term = PseudoTerm::new(parser.screen());
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.size());
            })
            .unwrap();
        terminal.backend().to_string()
    }

    #[test]
    fn empty_actions() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = vt100::Parser::new(24, 80, 0);
        parser.process(b" ");
        let pseudo_term = PseudoTerm::new(parser.screen());
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.size());
            })
            .unwrap();
        let view = terminal.backend().to_string();
        insta::assert_snapshot!(view);
    }

    #[test]
    fn simple_ls() {
        let stream = include_bytes!("../test/typescript/simple_ls.typescript");
        let view = snapshot_typescript(stream);
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
        let pseudo_term = PseudoTerm::new(parser.screen()).block(block);
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.size());
            })
            .unwrap();
        let view = terminal.backend().to_string();
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
