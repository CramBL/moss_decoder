use criterion::Criterion;

const BENCH_FILE_PATH: &str = "tests/moss_noise.raw";

pub fn decode_single_event(c: &mut Criterion) {
    let f = std::fs::read(std::path::PathBuf::from(BENCH_FILE_PATH)).unwrap();

    let mut group = c.benchmark_group("decode single event");
    {
        group.bench_function("decode event", |b| {
            b.iter(|| moss_decoder::decode_event(&f))
        });
        group.bench_function("decode event noexcept", |b| {
            b.iter(|| moss_decoder::decode_event_noexcept(&f))
        });
        group.bench_function("decode event fsm alt", |b| {
            b.iter(|| moss_decoder::slower_impls::decode_event_fsm_alt(&f))
        });
        group.bench_function("decode event fsm iterator", |b| {
            b.iter(|| moss_decoder::decode_event_fsm(&f))
        });
    }
    group.finish();
}
