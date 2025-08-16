# Crate: ftsim-cli

This crate provides the command-line interface (CLI) and main entry point for the FTSim application.

## Purpose

`ftsim-cli` is the user-facing executable. Its responsibilities include:

-   **Argument Parsing:** Defines and parses all command-line arguments and subcommands using the `clap` crate (e.g., `ftsim run --scenario ...`).
-   **Component Wiring:** It is responsible for instantiating all the core components of the simulator—the engine, the world, the telemetry bus, the protocol instances, and the TUI—and connecting them correctly.
-   **Configuration:** It loads scenario files, applies command-line overrides (like the RNG seed), and sets up the logging and metrics sinks.
-   **Protocol Registry:** It contains the static registry of all available protocols that can be run in the simulator.
-   **Execution:** It kicks off the simulation and, if enabled, the TUI event loop.

## Subcommands

-   `run`: The primary command to execute a simulation based on a scenario file.
-   `list-protocols`: Introspects the protocol registry and lists the available protocols and their associated tags.
-   `validate`: Parses and validates a scenario file for correctness without running it.
