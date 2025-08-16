# Crate: ftsim-modelcheck

This crate is designated for model-checking and exhaustive concurrency testing of FTSim components using tools like `loom` and `shuttle`.

## Purpose

While the core simulator is single-threaded and deterministic, some components (like a potential file-based `Store` or lock-free data structures for telemetry) might involve concurrency. This crate provides a dedicated space to write tests that systematically explore the state space of these concurrent components.

-   **`loom`:** Used for testing the internal invariants of concurrent data structures.
-   **`shuttle`:** Used for exploring the interleavings of events in a small-scale simulation to find bugs in protocol logic under different schedules.
