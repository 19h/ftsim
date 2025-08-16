//! # ftsim-engine::store
//!
//! The storage subsystem. This module provides a trait-based abstraction for
//! persistent storage, along with several implementations:
//! - `MemStore`: A simple, deterministic in-memory store.
//! - `FaultyStoreView`: A wrapper that injects storage failures around another store view.

mod faulty;
mod mem;
mod r#trait;

pub use faulty::{FaultyStoreView, StoreFaultModel};
pub use mem::MemStore;
pub use r#trait::{Store, StoreView};
