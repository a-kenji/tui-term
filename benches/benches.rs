use criterion::{black_box, criterion_group, criterion_main, Criterion};
use once_cell::sync::Lazy;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use termwiz::escape::Action;
use tui_term::pseudo_term::wezterm_action::PseudoTerm;

static SIMPLE_LS_ACTIONS: Lazy<Vec<Action>> = Lazy::new(|| {
    let simple_ls = include_bytes!("../test/typescript/simple_ls.typescript");
    let mut parser = termwiz::escape::parser::Parser::new();
    parser.parse_as_vec(simple_ls)
});

fn simple_ls() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let pseudo_term = PseudoTerm::new(&SIMPLE_LS_ACTIONS);
    terminal
        .draw(|f| {
            f.render_widget(pseudo_term, f.size());
        })
        .unwrap();
    let _view = terminal.backend().to_string();
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("simple ls", |b| b.iter(simple_ls));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
