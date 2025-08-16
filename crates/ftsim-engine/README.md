# Crate: ftsim-engine

This crate is the heart of the FTSim simulator. It contains all the logic for running deterministic, discrete-event simulations of distributed systems.

## Purpose

The engine is responsible for managing the entire state of a simulation, including:

-   **Time Management:** A master clock (`SimTime`) that advances based on scheduled events.
-   **Event Queue:** A priority queue that holds all future events, ensuring they are processed in the correct, deterministic order.
-   **World State:** A representation of the simulated environment, containing all nodes, the network topology, and storage backends.
-   **Deterministic RNG:** A single, master `ChaCha20Rng` instance that is the sole source of randomness for all probabilistic events (e.g., network drops, delays, election timeouts), ensuring reproducibility.
-   **Fault Injection:** Logic for applying the fault models defined in scenarios to the network and storage layers.
-   **Protocol Hosting:** It hosts and drives the user-defined protocols (via the `ProtocolDyn` trait object), providing them with a sandboxed context (`ProtoCtx`) to interact with the simulated world.
-   **Telemetry:** A comprehensive system for emitting structured logs, metrics, and state snapshots for observability.

## Core Components

-   **`sim.rs`:** Contains the main `Simulation` struct and the event loop logic (`step`, `run`). This is the central coordinator.
-   **`events.rs`:** Defines the `Event` enum, which represents all possible actions that can occur in the simulation (e.g., message delivery, timer firing, fault injection).
-   **`world.rs`:** Defines the `World` struct, which aggregates the nodes and the network.
-   **`net/`:** The network subsystem. It models the topology (`petgraph`), link properties, and applies fault models like delay, drop, duplication, and partitioning.
-   **`node/`:** The node runtime. It encapsulates a protocol instance, its storage, and its timers. It's responsible for handling events and dispatching them to the protocol logic.
-   **`store/`:** The storage subsystem. It provides a trait-based API for persistent storage and includes in-memory and faulty wrapper implementations.
-   **`telemetry/`:** The observability pipeline. It includes a `tracing` layer for contextual logs, a metrics sink, and a snapshot generator for the TUI.
-   **`scenario/`:** The scenario handler. It loads scenario files and schedules the specified directives as simulation events.

The engine is designed to be completely deterministic and self-contained. It has no concept of wall-clock time and performs no real I/O, except for logging and telemetry sinks when configured.
