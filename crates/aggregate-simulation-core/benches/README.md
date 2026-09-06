# Simulation performance

Run from the workspace root:

```sh
cargo bench -p aggregate-simulation-core --bench simulation_performance --locked -- --noplot
```

Criterion measures daily simulation, snapshot creation and country indexing separately at 128, 4,096 and 40,000 provinces. Fixtures use one population group and one facility per province, deterministic IDs and no map assets. World construction is outside the timed section; snapshot/index measurements include dropping the produced value. These are CPU benchmarks, not rendered frame-time measurements.

For a local comparison on the same machine and Rust toolchain:

```sh
cargo bench -p aggregate-simulation-core --bench simulation_performance -- --save-baseline before
# Change the implementation, keeping the benchmark fixture unchanged.
cargo bench -p aggregate-simulation-core --bench simulation_performance -- --baseline before
```

The opt-in elapsed-time workload pre-advances a fixed 64-province world by 0, 365 or 36,500 days before measurement. Each measurement continues advancing that world, so labels describe its starting age. It prints entity counts, command history length and serialized save size before measuring. Enable it in PowerShell with:

```powershell
$env:AGGREGATE_BENCH_LONG_RUN = '1'
cargo bench -p aggregate-simulation-core --bench simulation_performance --locked -- elapsed_years --noplot
Remove-Item Env:AGGREGATE_BENCH_LONG_RUN
```

This workload detects age-related overhead with fixed entity counts. It does not establish performance under population fragmentation, facility growth, large command histories, many enabled programs or peak memory pressure. Add representative workloads as those behaviors are implemented.

The `simulation performance` workflow runs on relevant pull requests and manual dispatch. The 100-year workload is selectable on manual dispatch. It records environment metadata, console output and Criterion reports as a 30-day artifact. The workflow currently reports measurements without a regression gate or automatic base-commit comparison; shared-runner noise must be characterized before introducing a blocking threshold. `cargo bench` does not replace correctness tests.

For actual UI responsiveness, Windows native tests cover the imported map and a Russia session with more than 40,000 land provinces:

```sh
cargo test -p aggregate-client native_large_world_capture -- --ignored --test-threads=1
cargo test -p aggregate-client frames_continue_and_committed_state_stays_stable_while_worker_waits
```

Worker diagnostics use `aggregate_client::simulation_performance=debug`: calculation, snapshot/index construction and main-thread snapshot application have separate timings. Native tests establish that frames continue and province lists are bounded; they do not enforce an FPS budget.
