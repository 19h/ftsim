//! # ftsim-engine::store::trait
//!
//! Defines the core traits for the storage subsystem, `Store` and `StoreView`.
//! This abstraction allows different storage backends (in-memory, file-based,
//! faulty) to be used interchangeably.

use ftsim_proto::api::StoreView as ProtoStoreView;

/// The main trait for a storage backend. It must be `Send` to be used in nodes.
pub trait Store: Send {
    /// Provides a view into the store, which is what protocols interact with.
    fn as_view(&mut self) -> &mut dyn StoreView;
}

/// A trait that combines the protocol-facing `StoreView` with engine-side requirements.
pub trait StoreView: ProtoStoreView {}

// Blanket implementation to automatically make any `ProtoStoreView` a `StoreView`.
impl<T: ProtoStoreView> StoreView for T {}
