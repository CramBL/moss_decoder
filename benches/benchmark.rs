use criterion::{criterion_group, criterion_main};

mod decode_from_file_bench;

criterion_group!(benches, decode_from_file_bench::decode_from_file,);
criterion_main!(benches);
