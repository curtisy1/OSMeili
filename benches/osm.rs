use criterion::{criterion_group, criterion_main, Criterion};
use osm_pbf2json::{extract_roads, filter, process};
use std::fs::File;
use std::io::{Result, Write};

struct MockWriter;

impl Write for MockWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Ok(buf.len())
    }
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

pub fn process_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("alexanderplatz");
    group.sample_size(10);
    let groups = filter::parse("amenity".to_string());
    group.bench_function("process", |b| {
        b.iter(|| {
            let file = File::open("./tests/data/alexanderplatz.pbf").unwrap();
            let mut writer = MockWriter;
            process(file, &mut writer, &groups).unwrap();
        })
    });
    group.finish();
}

pub fn extract_roads_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("alexanderplatz");
    group.sample_size(10);
    group.bench_function("extract_roads", |b| {
        b.iter(|| {
            let file = File::open("./tests/data/alexanderplatz.pbf").unwrap();
            let mut writer = MockWriter;
            extract_roads(file, &mut writer, false, None).unwrap();
        })
    });
    group.finish();
}

criterion_group!(benches, process_bench, extract_roads_bench);
criterion_main!(benches);
