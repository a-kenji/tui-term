use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use termwiz::escape::Action;

use crate::pseudo_term::PseudoTermState;

pub struct PseudoTerm<'a> {
    actions: &'a Vec<Action>,
}

impl<'a> PseudoTerm<'a> {
    pub fn new(actions: &'a Vec<Action>) -> Self {
        PseudoTerm { actions }
    }
}

impl Widget for PseudoTerm<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = PseudoTermState::default();
        state.handle_actions(self.actions, area, buf);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use super::*;

    #[test]
    fn empty_actions() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let actions = vec![];
        let pseudo_term = PseudoTerm::new(&actions);
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
        let backend = TestBackend::new(80, 24);
        let simple_ls = include_bytes!("../../test/typescript/simple_ls.typescript");
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = termwiz::escape::parser::Parser::new();
        let actions = parser.parse_as_vec(simple_ls);
        let pseudo_term = PseudoTerm::new(&actions);
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
        let backend = TestBackend::new(80, 24);
        let simple_ls = include_bytes!("../../test/typescript/vttest_02_01.typescript");
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = termwiz::escape::parser::Parser::new();
        let actions = parser.parse_as_vec(simple_ls);
        let pseudo_term = PseudoTerm::new(&actions);
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.size());
            })
            .unwrap();
        let view = terminal.backend().to_string();
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_02() {
        let backend = TestBackend::new(80, 24);
        let simple_ls = include_bytes!("../../test/typescript/vttest_02_02.typescript");
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = termwiz::escape::parser::Parser::new();
        let actions = parser.parse_as_vec(simple_ls);
        let pseudo_term = PseudoTerm::new(&actions);
        terminal
            .draw(|f| {
                f.render_widget(pseudo_term, f.size());
            })
            .unwrap();
        let view = terminal.backend().to_string();
        insta::assert_snapshot!(view);
    }
}
