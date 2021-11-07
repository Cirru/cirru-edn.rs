use criterion::{criterion_group, criterion_main, Criterion};
use std::fs;

use cirru_edn::parse;

fn criterion_benchmark(c: &mut Criterion) {
  let large_demo = "/Users/chen/repo/calcit-lang/editor/compact.cirru";
  let content = fs::read_to_string(large_demo).unwrap();

  c.bench_function("parse", |b| {
    b.iter(|| {
      let _ = parse(&content);
    })
  });

  let data = parse(&content).unwrap();

  c.bench_function("parse", |b| {
    b.iter(|| {
      let _ = cirru_edn::format(&data, true);
    })
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
