//! # ftsim-cli::commands::run
//!
//! Implements the `run` subcommand.

use crate::{
    args::RunOpts,
    logging::{HeadlessFormatter, SimulationFormatter},
    wiring::{build_world, finalize_world_setup, get_seed},
};
use anyhow::Result;
use ftsim_engine::{prelude::*, scenario::load_and_schedule, telemetry::tracing_layer::SimContextLayer};
use std::fs;
use tracing_subscriber::prelude::*;

pub fn exec(opts: RunOpts) -> Result<()> {
    // 1. Parse scenario ONCE
    let content = fs::read_to_string(&opts.scenario)?;
    let scenario: Scenario = match opts.scenario.extension().and_then(|s| s.to_str()) {
        Some("yaml") | Some("yml") => serde_yaml::from_str(&content)?,
        Some("toml") => toml::from_str(&content)?,
        _ => return Err(anyhow::anyhow!("Unsupported scenario file extension")),
    };
    scenario.validate().map_err(|e| anyhow::anyhow!(e))?;

    let seed = get_seed(opts.seed, scenario.seed);
    println!("Running scenario '{}' with seed: {}", scenario.name, seed);

    // 2. Build and finalize the world
    let mut world = build_world(&scenario)?;
    finalize_world_setup(&mut world);
    let num_nodes = world.nodes.len();

    // 3. Setup Telemetry and Control Channels
    let (snapshot_tx, snapshot_rx) = crossbeam_channel::unbounded();
    let (control_tx, control_rx) = crossbeam_channel::unbounded();
    let telemetry = TelemetryBus::new(snapshot_tx, num_nodes);
    let sim_context_layer = SimContextLayer::new(&telemetry);
    
    // Setup enhanced logging based on headless mode
    if opts.headless {
        // Use simplified formatter for headless mode
        tracing_subscriber::registry()
            .with(sim_context_layer)
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(HeadlessFormatter)
                    .with_ansi(true)
            )
            .with(tracing_subscriber::EnvFilter::from_default_env().add_directive("ftsim=info".parse().unwrap()))
            .init();
        
        println!("\n🎮 Starting FTSim headless execution...");
        println!("📊 Scenario: {}", scenario.name);
        println!("🎲 Seed: {}", seed);
        println!("⚙️  Nodes: {}", num_nodes);
        println!("{}", "=".repeat(60));
    } else {
        // Use detailed formatter for interactive/TUI mode
        tracing_subscriber::registry()
            .with(sim_context_layer)
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(SimulationFormatter::new())
                    .with_ansi(true)
            )
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .init();
    }

    // 4. Setup TUI (feature-gated)
    let use_tui = !opts.headless;

    #[cfg(feature = "tui")]
    let tui_handle = if use_tui {
        let control_tx_clone = control_tx.clone();
        Some(std::thread::spawn(move || {
            ftsim_tui::run_tui(snapshot_rx, control_tx_clone).expect("TUI failed");
        }))
    } else {
        None
    };

    #[cfg(not(feature = "tui"))]
    let tui_handle: Option<std::thread::JoinHandle<()>> = {
        if use_tui {
            println!("Warning: TUI requested but 'tui' feature is not enabled. Running headless.");
        }
        None
    };

    // 5. Create and run the simulation
    let mut sim = Simulation::new(seed, world, telemetry);
    sim.set_control_channel(control_rx);
    sim.init();
    load_and_schedule(&mut sim, &scenario)?;

    if use_tui && tui_handle.is_some() {
        sim.schedule_at(0, Event::UiSnapshotTick, EventDiscriminant::ui());
    }

    if let Some(stop_at_ms) = opts.stop_at {
        sim.run_until(sim_from_ms(stop_at_ms));
    } else if let Some(stop_at_ns) = scenario.stop_at {
        sim.run_until(stop_at_ns);
    } else {
        sim.run();
    }

    // 6. Shutdown and Summary
    if opts.headless {
        println!("{}", "=".repeat(60));
        println!("🏁 Simulation completed successfully!");
        
        // Get final snapshot for summary
        let final_snapshot = sim.telemetry().build_snapshot(&sim.world(), sim.now());
        println!("📈 Final Metrics:");
        println!("   • Messages Sent: {}", final_snapshot.metrics.messages_sent);
        println!("   • Messages Delivered: {}", final_snapshot.metrics.messages_delivered);
        println!("   • Timers Fired: {}", final_snapshot.metrics.timers_fired);
        println!("   • Faults Injected: {}", final_snapshot.metrics.faults_injected);
        
        println!("\n🏷️  Final Node States:");
        for node_snap in final_snapshot.nodes {
            let role = node_snap.custom.get("role")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let data_entries = node_snap.custom.get("data_entries")
                .and_then(|v| v.as_str())
                .unwrap_or("0");
            println!("   • Node {}: {} [{} status] - {} data entries", 
                     node_snap.id, role, format!("{:?}", node_snap.status).to_lowercase(), data_entries);
        }
    }

    if let Some(handle) = tui_handle {
        // In a real app, we'd signal the TUI to shut down.
        // For now, the user quits with 'q'. We'll let the process exit.
        // handle.join().expect("TUI thread panicked");
    }

    Ok(())
}
