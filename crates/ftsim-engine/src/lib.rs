//! # ftsim-engine
//!
//! The core of the FTSim simulator. This crate contains the main event loop,
//! world state management, network and storage models, fault injection logic,
//! and the telemetry pipeline.

// NOTE: Using unsafe code for performance-critical borrow checker workarounds
// All unsafe usage is carefully documented and limited to specific patterns

// Public modules, re-exporting key types for users of the engine.
pub mod control;
pub mod events;
pub mod ids;
pub mod net;
pub mod node;
pub mod prelude;
pub mod rng;
pub mod scenario;
pub mod sim;
pub mod store;
pub mod telemetry;
pub mod world;

// Internal-only modules
mod errors;
