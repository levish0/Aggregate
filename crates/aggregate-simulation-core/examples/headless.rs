use aggregate_simulation_core::Simulation;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut simulation = Simulation::default();
    for _ in 0..100 {
        simulation.step()?;
    }
    println!(
        "Headless simulation clock: {} logical ticks",
        simulation.clock().tick()
    );
    Ok(())
}
