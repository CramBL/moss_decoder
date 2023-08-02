use criterion::Criterion;

const BENCH_FILE_PATH: &str = "tests/test-data/moss_noise.raw";

pub fn decode_multiple_events(c: &mut Criterion) {
    let f = std::fs::read(std::path::PathBuf::from(BENCH_FILE_PATH)).unwrap();

    let mut group = c.benchmark_group("decode_multiple_events_bench");
    {
        group.bench_function("fsm iterator", |b| {
            b.iter(|| moss_decoder::decode_all_events(&f))
        });
    }
    group.finish();
}
