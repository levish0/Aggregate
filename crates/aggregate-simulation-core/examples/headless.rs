use aggregate_scenario::{load_scenario, parse_scenario};
use aggregate_simulation_core::{RecordedCommand, Simulation};
use std::{env, error::Error, fs};

const EXAMPLE_SCENARIO: &str = include_str!("../../../scenarios/foundation.json");
const EXAMPLE_COMMANDS: &str = include_str!("../../../scenarios/foundation.commands.json");

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()),
        )
        .with_writer(std::io::stderr)
        .init();
    let arguments: Vec<_> = env::args().skip(1).collect();
    if arguments
        .first()
        .is_some_and(|argument| argument == "--help")
    {
        println!("headless [scenario.json] [commands.json] [days=10] [save.json]");
        println!(
            "With no arguments, runs the bundled artificial scenario and construction command."
        );
        return Ok(());
    }
    if arguments.len() > 4 {
        return Err("expected at most four arguments; use --help".into());
    }
    let scenario = if let Some(path) = arguments.first() {
        load_scenario(path)?
    } else {
        parse_scenario(EXAMPLE_SCENARIO)?
    };
    let commands: Vec<RecordedCommand> = if let Some(path) = arguments.get(1) {
        serde_json::from_str(&fs::read_to_string(path)?)?
    } else if arguments.is_empty() {
        serde_json::from_str(EXAMPLE_COMMANDS)?
    } else {
        Vec::new()
    };
    let days: u64 = arguments
        .get(2)
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(10);
    println!("{} — native Rust rules, artificial fixture", scenario.name);
    let mut simulation = Simulation::from_scenario_with_programs(scenario.clone(), economy_programs())?;
    for record in &commands {
        if record.day < simulation.clock().day() || record.day > days {
            return Err("command day outside the requested ordered run".into());
        }
        while simulation.clock().day() < record.day {
            advance(&mut simulation)?;
        }
        let result = simulation.execute(record.command.clone())?;
        if result.sequence != record.sequence {
            return Err("non-consecutive command sequence".into());
        }
        println!("command {}: {:?}", result.sequence, result.event);
    }
    while simulation.clock().day() < days {
        advance(&mut simulation)?;
    }
    for province in simulation.snapshot().provinces {
        println!("{} stockpile: {:?}", province.name, province.stockpile);
    }
    let expected_hash = simulation.state_hash()?;
    let saved = simulation.save_json()?;
    let mut restored = Simulation::from_save_json_with_programs(&saved, economy_programs())?;
    let mut replayed = Simulation::replay_with_programs(scenario, &commands, days, economy_programs())?;
    if restored.state_hash()? != expected_hash || replayed.state_hash()? != expected_hash {
        return Err("save/replay state mismatch".into());
    }
    println!("Save/load and command replay match: {expected_hash}");
    if let Some(path) = arguments.get(3) {
        fs::write(path, saved)?;
        println!("Saved to {path}");
    }
    Ok(())
}

fn advance(simulation: &mut Simulation) -> Result<(), Box<dyn Error>> {
    let report = simulation.step()?;
    for province in &report.provinces {
        println!(
            "day {} | {} | workers {}/{} production + {} construction | food shortfall {}",
            report.day,
            province.province,
            province.production_workers,
            province.available_workers,
            province.construction_workers,
            province.food_shortfall
        );
    }
    for event in &report.events {
        println!("  {event:?}");
    }
    Ok(())
}

fn economy_programs() -> Vec<std::sync::Arc<dyn aggregate_programs::SimulationProgram>> {
    vec![std::sync::Arc::new(aggregate_economy::EconomyProgram)]
}
