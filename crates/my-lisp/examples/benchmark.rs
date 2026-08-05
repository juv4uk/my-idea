use my_lisp::{eval_program, parse, Session};
use std::{fs, hint::black_box, time::Instant};

const CASES: &[(&str, &str)] = &[
    ("arithmetic", "benchmarks/arithmetic.my"),
    ("lists", "benchmarks/lists.my"),
    ("recursion", "benchmarks/recursion.my"),
    ("closures", "benchmarks/closures.my"),
];

fn measure(iterations: usize, mut operation: impl FnMut()) -> f64 {
    for _ in 0..50 {
        operation();
    }
    let started = Instant::now();
    for _ in 0..iterations {
        operation();
    }
    started.elapsed().as_nanos() as f64 / iterations as f64
}

fn main() {
    let iterations = std::env::var("MY_LISP_BENCH_ITERATIONS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1_000);
    let parser_source = fs::read_to_string("benchmarks/parser.my").expect("read parser benchmark");
    let parser_ns = measure(iterations, || {
        black_box(parse(black_box(&parser_source)).expect("parse benchmark"));
    });
    println!("BENCH_RESULT\trust\tparser\t{parser_ns:.2}");

    for (name, path) in CASES {
        let source = fs::read_to_string(path).expect("read evaluation benchmark");
        let ns = measure(iterations, || {
            let mut session = Session::default();
            black_box(eval_program(black_box(&source), &mut session).expect("evaluate benchmark"));
        });
        println!("BENCH_RESULT\trust\t{name}\t{ns:.2}");
    }
}
