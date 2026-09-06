mod support;
use aggregate_world::WorldSnapshotIndex;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::{hint::black_box, time::Duration};

fn world_scale(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("world_scale");
    group.sample_size(10).warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(2));
    for count in [128, 4096, 40_000] {
        let mut simulation = support::simulation(support::world(count));
        group.throughput(Throughput::Elements(count as u64));
        group.bench_function(BenchmarkId::new("day", count), |bench| {
            bench.iter(|| black_box(simulation.step().unwrap()));
        });
        group.bench_function(BenchmarkId::new("snapshot", count), |bench| {
            bench.iter(|| black_box(simulation.snapshot()));
        });
        let snapshot = simulation.snapshot();
        group.bench_function(BenchmarkId::new("country_index", count), |bench| {
            bench.iter(|| black_box(WorldSnapshotIndex::build(black_box(&snapshot))));
        });
    }
    group.finish();
}

fn long_run(criterion: &mut Criterion) {
    if std::env::var("AGGREGATE_BENCH_LONG_RUN").as_deref() != Ok("1") { return; }
    let mut group = criterion.benchmark_group("elapsed_years_fixed_64_provinces");
    group.sample_size(10).warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(2));
    for age in [0, 365, 36_500] {
        let mut simulation = support::simulation(support::world(64));
        for _ in 0..age { simulation.step().unwrap(); }
        let snapshot = simulation.snapshot();
        println!("LONG_RUN day={age} provinces={} groups={} facilities={} commands={} save_bytes={}", snapshot.provinces.len(), snapshot.population_groups.len(), snapshot.facilities.len(), simulation.command_log().len(), simulation.save_json().unwrap().len());
        group.bench_function(BenchmarkId::new("day", age), |bench| bench.iter(|| black_box(simulation.step().unwrap())));
    }
    group.finish();
}

criterion_group!(benches, world_scale, long_run);
criterion_main!(benches);
