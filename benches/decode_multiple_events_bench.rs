use criterion::Criterion;

const BENCH_FILE_PATH: &str = "tests/moss_noise.raw";

pub fn decode_multiple_events(c: &mut Criterion) {
    let f = std::fs::read(std::path::PathBuf::from(BENCH_FILE_PATH)).unwrap();

    let mut group = c.benchmark_group("decode_multiple_events_bench");
    {
        group.bench_function("default", |b| {
            b.iter(|| moss_decoder::decode_multiple_events(&f))
        });
        group.bench_function("fsm", |b| {
            b.iter(|| moss_decoder::slower_impls::decode_multiple_events_fsm(&f))
        });
        group.bench_function("fsm alternative", |b| {
            b.iter(|| moss_decoder::slower_impls::decode_multiple_events_fsm_alt(&f))
        });
        group.bench_function("fsm iterator", |b| {
            b.iter(|| moss_decoder::decode_multiple_events_fsm(&f))
        });
    }
    group.finish();
}
