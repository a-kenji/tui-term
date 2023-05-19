use criterion::{black_box, criterion_group, criterion_main, Criterion};

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

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
