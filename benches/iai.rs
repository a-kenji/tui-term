use once_cell::sync::Lazy;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use tui_term::widget::PseudoTerm;
use vt100::Screen;

static SIMPLE_LS_ACTIONS: Lazy<Screen> = Lazy::new(|| {
    let stream = include_bytes!("../test/typescript/simple_ls.typescript");
    let mut parser = vt100::Parser::new(24, 80, 0);
    parser.process(stream);
    parser.screen().clone()
});

static VTTEST_02_01: Lazy<Screen> = Lazy::new(|| {
    let stream = include_bytes!("../test/typescript/vttest_02_01.typescript");
    let mut parser = vt100::Parser::new(24, 80, 0);
    parser.process(stream);
    parser.screen().clone()
});

static VTTEST_02_02: Lazy<Screen> = Lazy::new(|| {
    let stream = include_bytes!("../test/typescript/vttest_02_02.typescript");
    let mut parser = vt100::Parser::new(24, 80, 0);
    parser.process(stream);
    parser.screen().clone()
});

static VTTEST_02_03: Lazy<Screen> = Lazy::new(|| {
    let stream = include_bytes!("../test/typescript/vttest_02_03.typescript");
    let mut parser = vt100::Parser::new(24, 80, 0);
    parser.process(stream);
    parser.screen().clone()
});

static VTTEST_02_04: Lazy<Screen> = Lazy::new(|| {
    let stream = include_bytes!("../test/typescript/vttest_02_04.typescript");
    let mut parser = vt100::Parser::new(24, 80, 0);
    parser.process(stream);
    parser.screen().clone()
});

static VTTEST_02_05: Lazy<Screen> = Lazy::new(|| {
    let stream = include_bytes!("../test/typescript/vttest_02_05.typescript");
    let mut parser = vt100::Parser::new(24, 80, 0);
    parser.process(stream);
    parser.screen().clone()
});

static VTTEST_02_06: Lazy<Screen> = Lazy::new(|| {
    let stream = include_bytes!("../test/typescript/vttest_02_06.typescript");
    let mut parser = vt100::Parser::new(24, 80, 0);
    parser.process(stream);
    parser.screen().clone()
});

static VTTEST_02_07: Lazy<Screen> = Lazy::new(|| {
    let stream = include_bytes!("../test/typescript/vttest_02_07.typescript");
    let mut parser = vt100::Parser::new(24, 80, 0);
    parser.process(stream);
    parser.screen().clone()
});

static VTTEST_02_08: Lazy<Screen> = Lazy::new(|| {
    let stream = include_bytes!("../test/typescript/vttest_02_08.typescript");
    let mut parser = vt100::Parser::new(24, 80, 0);
    parser.process(stream);
    parser.screen().clone()
});

static VTTEST_02_09: Lazy<Screen> = Lazy::new(|| {
    let stream = include_bytes!("../test/typescript/vttest_02_09.typescript");
    let mut parser = vt100::Parser::new(24, 80, 0);
    parser.process(stream);
    parser.screen().clone()
});

static VTTEST_02_10: Lazy<Screen> = Lazy::new(|| {
    let stream = include_bytes!("../test/typescript/vttest_02_10.typescript");
    let mut parser = vt100::Parser::new(24, 80, 0);
    parser.process(stream);
    parser.screen().clone()
});

static VTTEST_02_11: Lazy<Screen> = Lazy::new(|| {
    let stream = include_bytes!("../test/typescript/vttest_02_11.typescript");
    let mut parser = vt100::Parser::new(24, 80, 0);
    parser.process(stream);
    parser.screen().clone()
});

static VTTEST_02_12: Lazy<Screen> = Lazy::new(|| {
    let stream = include_bytes!("../test/typescript/vttest_02_12.typescript");
    let mut parser = vt100::Parser::new(24, 80, 0);
    parser.process(stream);
    parser.screen().clone()
});

static VTTEST_02_13: Lazy<Screen> = Lazy::new(|| {
    let stream = include_bytes!("../test/typescript/vttest_02_13.typescript");
    let mut parser = vt100::Parser::new(24, 80, 0);
    parser.process(stream);
    parser.screen().clone()
});

static VTTEST_02_14: Lazy<Screen> = Lazy::new(|| {
    let stream = include_bytes!("../test/typescript/vttest_02_14.typescript");
    let mut parser = vt100::Parser::new(24, 80, 0);
    parser.process(stream);
    parser.screen().clone()
});

static VTTEST_02_15: Lazy<Screen> = Lazy::new(|| {
    let stream = include_bytes!("../test/typescript/vttest_02_15.typescript");
    let mut parser = vt100::Parser::new(24, 80, 0);
    parser.process(stream);
    parser.screen().clone()
});

#[inline]
fn render_typescript(screen: &Screen) {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let pseudo_term = PseudoTerm::new(screen);
    terminal
        .draw(|f| {
            f.render_widget(pseudo_term, f.size());
        })
        .unwrap();
    let _view = terminal.backend().to_string();
}

#[inline]
fn simple_ls() {
    render_typescript(&SIMPLE_LS_ACTIONS)
}

#[inline]
fn vttest_02_01() {
    render_typescript(&VTTEST_02_01)
}

#[inline]
fn vttest_02_02() {
    render_typescript(&VTTEST_02_02)
}

#[inline]
fn vttest_02_03() {
    render_typescript(&VTTEST_02_03)
}

#[inline]
fn vttest_02_04() {
    render_typescript(&VTTEST_02_04)
}

#[inline]
fn vttest_02_05() {
    render_typescript(&VTTEST_02_05)
}

#[inline]
fn vttest_02_06() {
    render_typescript(&VTTEST_02_06)
}

#[inline]
fn vttest_02_07() {
    render_typescript(&VTTEST_02_07)
}

#[inline]
fn vttest_02_08() {
    render_typescript(&VTTEST_02_08)
}

#[inline]
fn vttest_02_09() {
    render_typescript(&VTTEST_02_09)
}

#[inline]
fn vttest_02_10() {
    render_typescript(&VTTEST_02_10)
}

#[inline]
fn vttest_02_11() {
    render_typescript(&VTTEST_02_11)
}

#[inline]
fn vttest_02_12() {
    render_typescript(&VTTEST_02_12)
}

#[inline]
fn vttest_02_13() {
    render_typescript(&VTTEST_02_13)
}

#[inline]
fn vttest_02_14() {
    render_typescript(&VTTEST_02_14)
}

#[inline]
fn vttest_02_15() {
    render_typescript(&VTTEST_02_15)
}

iai::main!(
    simple_ls,
    vttest_02_01,
    vttest_02_02,
    vttest_02_03,
    vttest_02_04,
    vttest_02_05,
    vttest_02_06,
    vttest_02_07,
    vttest_02_08,
    vttest_02_09,
    vttest_02_10,
    vttest_02_11,
    vttest_02_12,
    vttest_02_13,
    vttest_02_14,
    vttest_02_15,
);
