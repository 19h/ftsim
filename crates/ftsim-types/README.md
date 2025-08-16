# Crate: ftsim-types

This crate provides the foundational, shared data types used across the entire FTSim workspace.

## Purpose

The primary role of `ftsim-types` is to break dependency cycles. Crates like `ftsim-engine`, `ftsim-proto`, and `ftsim-cli` all need to agree on common structures for identification, time, network messages, and configuration. By placing these types in a central, stable, and dependency-free (within the workspace) crate, we ensure a clean and hierarchical dependency graph.

This crate is designed to be low-volatility. Changes here have a cascading effect on the entire project.

## Key Data Structures

-   **IDs (`id.rs`):** Simple type aliases for `NodeId`, `LinkId`, `TimerId`, etc., providing strong typing for identifiers throughout the simulation.
-   **Time (`time.rs`):** Defines `SimTime` (a `u128` in nanoseconds) and provides constants and safe arithmetic for handling simulation time.
-   **Envelope (`envelope.rs`):** The core message wrapper for all network communication, containing source, destination, payload, and metadata for tracing and fault injection.
-   **Errors (`errors.rs`):** A set of `thiserror`-based error enums for handling failures in a structured way.
-   **Configuration (`config.rs`, `scenario.rs`, `topology.rs`):** Strongly-typed `serde`-compatible structs that define the schema for scenario files (e.g., `Scenario`, `Directive`, `Action`). This allows for safe and easy parsing of YAML/TOML configuration.
-   **Metrics (`metrics.rs`):** Defines constants for metric names and labels, ensuring consistency in telemetry reporting.
