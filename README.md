# FTSim — Fault Tolerance Simulator

**FTSim** is a deterministic, discrete-event simulator for distributed systems, written in Rust. It is designed to provide a powerful, reproducible, and observable environment for developing, testing, and verifying the correctness of fault-tolerant algorithms and protocols.

The project's core mission is to enable developers to subject their distributed protocols to a wide range of realistic and adversarial conditions—including network partitions, message loss, clock skew, and storage failures—in a controlled and repeatable manner.

## Core Features

*   **Deterministic Core:** Given the same seed and scenario, FTSim guarantees bit-for-bit reproducible simulations, which is essential for debugging complex distributed behavior.
*   **Pluggable Protocols:** A clean and ergonomic trait-based API (`Protocol<M>`) allows developers to easily integrate their own algorithms (like Raft, Paxos, or custom protocols) into the simulator.
*   **Rich Failure Injection:** Scenarios can declaratively inject a wide array of failures, from simple message drops and network delays to complex partitions, node crashes, torn writes in storage, and even Byzantine behaviors.
*   **High-Signal Observability:** The simulator is built with observability as a first-class citizen.
    *   **Structured Logging:** All events are logged with rich context (simulation time, node IDs, trace IDs) in both human-readable and JSON formats.
    *   **Metrics:** Key performance and fault indicators are tracked via counters, gauges, and histograms.
    *   **Interactive TUI:** An optional terminal-based UI provides a real-time visualization of the cluster topology, node states, message queues, performance metrics, and event timelines.
*   **Scenario-Driven Experiments:** Simulations are defined in simple YAML or TOML files, allowing for easy versioning and sharing of experimental setups.

## Project Philosophy

FTSim is a **simulator**, not a production runtime. Its single-threaded, discrete-event core is optimized for control and reproducibility, not for wall-clock performance of the simulated protocol. This design choice allows for perfect control over time and the ordering of events, eliminating the non-determinism inherent in real-world distributed environments.

## Workspace Structure

The project is organized into a Cargo workspace with several specialized crates:

*   `ftsim-types`: Contains all shared data structures (IDs, Timestamps, Envelopes, Configuration schemas) used across the entire workspace.
*   `ftsim-proto`: Defines the SDK for implementing protocols, including the core `Protocol` trait and example implementations.
*   `ftsim-engine`: The heart of the simulator. It contains the event loop, world state, network and storage models, fault injection logic, and telemetry systems.
*   `ftsim-tui`: An optional, `ratatui`-based terminal user interface for interactive visualization and control of simulations.
*   `ftsim-cli`: The main binary entry point, responsible for parsing command-line arguments, loading scenarios, and wiring all the components together.

## Getting Started

1.  **Build the project:**
    cargo build --release --features tui
2.  **Run a scenario:**
    # Run with the interactive TUI
    cargo run --release --features tui -- run --scenario scenarios/raft_partition.yaml

    # Run in headless mode with JSON logging
    cargo run --release -- run --scenario scenarios/raft_partition.yaml --headless --log json
3.  **Explore available protocols:**
    cargo run --release -- list-protocols

This project is built according to a rigorous, authoritative specification to ensure correctness, maintainability, and a clear architectural vision.
