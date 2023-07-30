use criterion::{criterion_group, criterion_main};

mod decode_from_file_bench;
mod decode_multiple_events_bench;
mod decode_single_event_bench;

criterion_group!(
    benches,
    decode_from_file_bench::decode_from_file,
    decode_multiple_events_bench::decode_multiple_events,
    decode_single_event_bench::decode_single_event
);
criterion_main!(benches);
