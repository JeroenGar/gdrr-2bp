use std::hint::black_box;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::time::Duration;

use criterion::{BatchSize, Criterion, Throughput, criterion_group, criterion_main};
use gdrr_2bp::COST_COMPARATOR;
use gdrr_2bp::io::json_format::JsonInstance;
use gdrr_2bp::io::parser;
use gdrr_2bp::optimization::config::{Config, SheetValuationMode};
use gdrr_2bp::optimization::gdrr::GDRR;
use gdrr_2bp::optimization::sol_collectors::local_sol_collector::LocalSolCollector;

const N_ITERATIONS: usize = 50_000;
const LARGE_EXAMPLE: &str = include_str!("../examples/large_example_input.json");

fn ci_bench(c: &mut Criterion) {
    let config = Config {
        avg_nodes_removed: 6,
        blink_rate: 0.01,
        max_run_time: None,
        max_rr_iterations: Some(N_ITERATIONS),
        random_seed: Some(0),
        leftover_valuation_power: 2.0,
        history_length: 500,
        rotation_allowed: true,
        n_threads: 1,
        sheet_valuation_mode: SheetValuationMode::Area,
        max_stages: None,
    };
    let mut json_instance: JsonInstance = serde_json::from_str(LARGE_EXAMPLE).unwrap();
    let instance = Arc::new(parser::generate_instance(&mut json_instance, &config));

    let mut group = c.benchmark_group("gdrr_iterations");
    group.throughput(Throughput::Elements(N_ITERATIONS as u64));
    group.bench_function("large_example", |b| {
        b.iter_batched_ref(
            || {
                let (tx_sync, rx_sync) = channel();
                let (tx_solution_report, rx_solution_report) = channel();
                let collector = LocalSolCollector::new(
                    instance.clone(),
                    rx_sync,
                    tx_solution_report,
                    COST_COMPARATOR,
                );
                let solver = GDRR::new(&instance, &config, collector);
                (solver, tx_sync, rx_solution_report)
            },
            |(solver, _tx_sync, _rx_solution_report)| {
                solver.optimize();
                black_box(solver);
            },
            BatchSize::PerIteration,
        )
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3))
        .without_plots();
    targets = ci_bench
}
criterion_main!(benches);
