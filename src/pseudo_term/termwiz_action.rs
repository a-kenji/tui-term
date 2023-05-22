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

    fn snapshot_typescript(stream: &[u8]) -> String {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut parser = termwiz::escape::parser::Parser::new();
        let actions = parser.parse_as_vec(stream);
        let pseudo_term = PseudoTerm::new(&actions);
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
        let stream = include_bytes!("../../test/typescript/simple_ls.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    #[test]
    fn vttest_02_01() {
        let stream = include_bytes!("../../test/typescript/vttest_02_01.typescript");
        let view = snapshot_typescript(stream);
        insta::assert_snapshot!(view);
    }
    // #[test]
    // fn vttest_02_02() {
    //     let stream = include_bytes!("../../test/typescript/vttest_02_02.typescript");
    //     let view = snapshot_typescript(stream);
    //     insta::assert_snapshot!(view);
    // }
    // #[test]
    // fn vttest_02_03() {
    //     let stream = include_bytes!("../../test/typescript/vttest_02_03.typescript");
    //     let view = snapshot_typescript(stream);
    //     insta::assert_snapshot!(view);
    // }
    // #[test]
    // fn vttest_02_04() {
    //     let stream = include_bytes!("../../test/typescript/vttest_02_04.typescript");
    //     let view = snapshot_typescript(stream);
    //     insta::assert_snapshot!(view);
    // }
    // #[test]
    // fn vttest_02_05() {
    //     let stream = include_bytes!("../../test/typescript/vttest_02_05.typescript");
    //     let view = snapshot_typescript(stream);
    //     insta::assert_snapshot!(view);
    // }
    // #[test]
    // fn vttest_02_06() {
    //     let stream = include_bytes!("../../test/typescript/vttest_02_06.typescript");
    //     let view = snapshot_typescript(stream);
    //     insta::assert_snapshot!(view);
    // }
    // #[test]
    // fn vttest_02_07() {
    //     let stream = include_bytes!("../../test/typescript/vttest_02_07.typescript");
    //     let view = snapshot_typescript(stream);
    //     insta::assert_snapshot!(view);
    // }
    // #[test]
    // fn vttest_02_08() {
    //     let stream = include_bytes!("../../test/typescript/vttest_02_08.typescript");
    //     let view = snapshot_typescript(stream);
    //     insta::assert_snapshot!(view);
    // }
    // #[test]
    // fn vttest_02_09() {
    //     let stream = include_bytes!("../../test/typescript/vttest_02_09.typescript");
    //     let view = snapshot_typescript(stream);
    //     insta::assert_snapshot!(view);
    // }
    // #[test]
    // fn vttest_02_10() {
    //     let stream = include_bytes!("../../test/typescript/vttest_02_10.typescript");
    //     let view = snapshot_typescript(stream);
    //     insta::assert_snapshot!(view);
    // }
    // #[test]
    // fn vttest_02_11() {
    //     let stream = include_bytes!("../../test/typescript/vttest_02_11.typescript");
    //     let view = snapshot_typescript(stream);
    //     insta::assert_snapshot!(view);
    // }
    // #[test]
    // fn vttest_02_12() {
    //     let stream = include_bytes!("../../test/typescript/vttest_02_12.typescript");
    //     let view = snapshot_typescript(stream);
    //     insta::assert_snapshot!(view);
    // }
    // #[test]
    // fn vttest_02_13() {
    //     let stream = include_bytes!("../../test/typescript/vttest_02_13.typescript");
    //     let view = snapshot_typescript(stream);
    //     insta::assert_snapshot!(view);
    // }
    // #[test]
    // fn vttest_02_14() {
    //     let stream = include_bytes!("../../test/typescript/vttest_02_14.typescript");
    //     let view = snapshot_typescript(stream);
    //     insta::assert_snapshot!(view);
    // }
    // #[test]
    // fn vttest_02_15() {
    //     let stream = include_bytes!("../../test/typescript/vttest_02_15.typescript");
    //     let view = snapshot_typescript(stream);
    //     insta::assert_snapshot!(view);
    // }
}
