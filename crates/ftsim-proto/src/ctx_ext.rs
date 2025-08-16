//! # ftsim-proto::ctx_ext
//!
//! Defines the `Ctx<M>` struct, which is the primary, ergonomic API for
//! protocol authors. It wraps the engine's `ProtoCtx` trait object and
//! provides typed, convenient methods for common operations like sending
//! messages and setting timers.

use crate::api::{ProtoCtx, StoreView};
use ftsim_types::{
    envelope::ProtoTag,
    errors::CodecError,
    id::{NodeId, TimerId},
    time::SimTime,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, marker::PhantomData};

/// A typed context wrapper provided to `Protocol<M>` implementations.
pub struct Ctx<'a, M> {
    inner: &'a mut dyn ProtoCtx,
    proto_tag: ProtoTag,
    _p: PhantomData<M>,
}

impl<'a, M> Ctx<'a, M> {
    pub(crate) fn new(inner: &'a mut dyn ProtoCtx, proto_tag: ProtoTag) -> Self {
        Self {
            inner,
            proto_tag,
            _p: PhantomData,
        }
    }
}

impl<'a, M> Ctx<'a, M>
where
    M: Serialize + DeserializeOwned + Debug + Send + 'static,
{
    /// Sends a typed message to a specific destination node.
    /// The message will be serialized using `postcard`.
    pub fn send(&mut self, dst: NodeId, msg: &M) -> Result<(), CodecError> {
        let bytes = postcard::to_allocvec(msg)
            .map_err(|e| CodecError(format!("Serialization failed: {}", e)))?;
        self.inner.send_raw(dst, self.proto_tag, bytes.into());
        Ok(())
    }

    /// Broadcasts a typed message to all other nodes, with an optional filter.
    pub fn broadcast(
        &mut self,
        msg: &M,
        filter: Option<&dyn Fn(NodeId) -> bool>,
    ) -> Result<(), CodecError> {
        let bytes = postcard::to_allocvec(msg)
            .map_err(|e| CodecError(format!("Serialization failed: {}", e)))?;
        self.inner
            .broadcast_raw(self.proto_tag, bytes.into(), filter);
        Ok(())
    }

    /// Sets a timer that will fire after the specified duration.
    /// Returns a `TimerId` that can be used to cancel it.
    pub fn set_timer(&mut self, after: SimTime) -> TimerId {
        self.inner.set_timer(after)
    }

    /// Cancels a pending timer. Returns `true` if the timer was found and canceled.
    pub fn cancel_timer(&mut self, timer: TimerId) -> bool {
        self.inner.cancel_timer(timer)
    }

    /// Returns the current simulation time, adjusted for this node's clock skew.
    pub fn now(&self) -> SimTime {
        self.inner.now()
    }

    /// Returns the ID of the current node.
    pub fn node_id(&self) -> NodeId {
        self.inner.node_id()
    }

    /// Provides temporary mutable access to the node's persistent storage.
    pub fn store(&mut self) -> Box<dyn StoreView + '_> {
        self.inner.store()
    }

    /// Returns a deterministic `u64` from the simulation's master RNG.
    /// This MUST be used for any randomness required by the protocol (e.g., election timeouts).
    pub fn rng_u64(&mut self) -> u64 {
        self.inner.rng_u64()
    }

    /// Attaches a key-value pair to the current logging span.
    /// This is useful for exposing protocol-specific state to the TUI and logs.
    /// Example: `ctx.log_kv("role", "leader")`.
    pub fn log_kv(&mut self, key: &'static str, val: &str) {
        self.inner.log_kv(key, val);
    }

    /// Helper method to log serializable values by converting them to JSON strings.
    pub fn log_kv_json<T: Serialize>(&mut self, key: &'static str, val: &T) {
        if let Ok(json_str) = serde_json::to_string(val) {
            self.inner.log_kv(key, &json_str);
        }
    }
}
