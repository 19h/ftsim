# Crate: ftsim-tui

This crate provides the interactive terminal user interface (TUI) for FTSim, built using the `ratatui` and `crossterm` libraries.

## Purpose

The TUI's goal is to offer high-signal, real-time observability into a running simulation. It visualizes the complex state of the distributed system in an intuitive and interactive manner, allowing users to not only watch the simulation unfold but also to inject faults dynamically.

## Features

-   **Real-time Visualization:** Receives state `Snapshot`s from the simulation engine and renders them at a consistent frame rate.
-   **Interactive Control:** Allows the user to pause, resume, and single-step the simulation.
-   **Dynamic Fault Injection:** Provides keybindings to inject faults like node crashes, restarts, and network partitions on the fly.
-   **Comprehensive Layout:** The UI is divided into several panes as per the specification:
    -   A **Cluster Graph** showing node status and network connectivity.
    -   A **Node Status Grid** displaying protocol-specific state.
    -   A **Metrics Panel** with sparklines for key performance indicators.
    -   A **Timeline and Log Viewer** for observing the sequence of events and filtering logs.
-   **Decoupled Architecture:** The TUI runs in its own thread and communicates with the `ftsim-engine` via channels, ensuring that UI rendering does not block the simulation's deterministic execution.

## Implementation Details

-   **`app.rs`:** Contains the `App` struct, which holds the TUI's state (e.g., the latest snapshot, UI focus state).
-   **`ui/`:** The rendering module. `layout.rs` defines the main screen layout, and the `widgets/` subdirectory contains the rendering logic for each individual UI component.
-   **`input.rs`:** Handles keyboard input from the user and translates it into actions or control messages sent to the simulation engine.
-   **`lib.rs`:** Contains the main TUI event loop (`run_tui`) that initializes the terminal, handles events, draws the UI, and restores the terminal on exit.
