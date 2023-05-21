use criterion::{black_box, criterion_group, criterion_main, Criterion};
use once_cell::sync::Lazy;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use termwiz::escape::Action;
use tui_term::pseudo_term::termwiz_action::PseudoTerm;

static SIMPLE_LS_ACTIONS: Lazy<Vec<Action>> = Lazy::new(|| {
    let simple_ls = include_bytes!("../test/typescript/simple_ls.typescript");
    let mut parser = termwiz::escape::parser::Parser::new();
    parser.parse_as_vec(simple_ls)
});

static VTTEST_02_01: Lazy<Vec<Action>> = Lazy::new(|| {
    let simple_ls = include_bytes!("../test/typescript/vttest_02_01.typescript");
    let mut parser = termwiz::escape::parser::Parser::new();
    parser.parse_as_vec(simple_ls)
});

#[inline]
fn render_typescript(actions: Vec<Action>) {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let pseudo_term = PseudoTerm::new(&actions);
    terminal
        .draw(|f| {
            f.render_widget(pseudo_term, f.size());
        })
        .unwrap();
    let _view = terminal.backend().to_string();
}

#[inline]
fn simple_ls() {
    render_typescript(SIMPLE_LS_ACTIONS.to_vec())
}

#[inline]
fn vttest_02_01() {
    render_typescript(VTTEST_02_01.to_vec())
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("simple ls", |b| b.iter(simple_ls));
    c.bench_function("vttest_02_01", |b| b.iter(vttest_02_01));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
