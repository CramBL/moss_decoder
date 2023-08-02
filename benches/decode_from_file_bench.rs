use criterion::Criterion;

const BENCH_FILE_PATH: &str = "tests/test-data/moss_noise.raw";

pub fn decode_from_file(c: &mut Criterion) {
    let mut group = c.benchmark_group("decode_multiple_events_bench");
    {
        group.bench_function("fsm iterator decode_from_file_fsm()", |b| {
            b.iter(|| moss_decoder::decode_from_file(BENCH_FILE_PATH.into()))
        });
    }
    group.finish();
}
