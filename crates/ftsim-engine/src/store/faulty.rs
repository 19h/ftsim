//! # ftsim-engine::store::faulty
//!
//! A wrapper store that injects faults around an inner `Store` implementation.
//! It uses the master RNG to decide when to inject failures like I/O errors,
//! torn writes, or fsync failures, based on configured rates.

use crate::{prelude::*, sim::EngineCtx};
use ftsim_proto::api::{LogIndex, LogRecord, StoreView as ProtoStoreView};
use rand::Rng;

/// The configuration for fault injection on a store.
#[derive(Default, Clone, Copy)]
pub struct StoreFaultModel {
    pub fsync_fail_rate: f64,
    pub fsync_delay_rate: f64,
    pub write_error_rate: f64,
    pub read_error_rate: f64,
    pub torn_write_rate: f64,
    pub stale_read_rate: f64,
}

/// A temporary view that wraps a `StoreView` to inject faults deterministically.
/// It borrows the `EngineCtx` to get access to the master RNG for the duration
/// of a single event handler.
pub struct FaultyStoreView<'a, 'b> {
    inner: &'a mut dyn ProtoStoreView,
    model: &'a StoreFaultModel,
    ctx: &'a mut EngineCtx<'b>,
}

impl<'a, 'b> FaultyStoreView<'a, 'b> {
    pub fn new(
        inner: &'a mut dyn ProtoStoreView,
        model: &'a StoreFaultModel,
        ctx: &'a mut EngineCtx<'b>,
    ) -> Self {
        Self { inner, model, ctx }
    }
}

impl ProtoStoreView for FaultyStoreView<'_, '_> {
    fn append_log(&mut self, rec: LogRecord) -> Result<LogIndex, StoreError> {
        let node_id = self.ctx.node_id();

        // Check for write error fault
        if self.model.write_error_rate > 0.0 {
            let site = Box::leak(format!("store.append_log.write_error.node[{}]", node_id).into_boxed_str());
            if self.ctx.rng(site).gen_bool(self.model.write_error_rate) {
                tracing::warn!(%node_id, "Injecting write error in append_log");
                return Err(StoreError::FaultInjected);
            }
        }

        // Check for torn write fault (partial write)
        if self.model.torn_write_rate > 0.0 {
            let site = Box::leak(format!("store.append_log.torn_write.node[{}]", node_id).into_boxed_str());
            if self.ctx.rng(site).gen_bool(self.model.torn_write_rate) {
                tracing::warn!(%node_id, "Injecting torn write in append_log");
                // For torn writes, we could partially corrupt the record, but for simplicity,
                // we'll just return an error to indicate the write was incomplete
                return Err(StoreError::FaultInjected);
            }
        }

        self.inner.append_log(rec)
    }

    fn read_log(&mut self, idx: LogIndex) -> Result<Option<LogRecord>, StoreError> {
        let node_id = self.ctx.node_id();

        // Check for read error fault
        if self.model.read_error_rate > 0.0 {
            let site = Box::leak(format!("store.read_log.read_error.node[{}]", node_id).into_boxed_str());
            if self.ctx.rng(site).gen_bool(self.model.read_error_rate) {
                tracing::warn!(%node_id, "Injecting read error in read_log");
                return Err(StoreError::FaultInjected);
            }
        }

        // Check for stale read fault (return outdated data)
        if self.model.stale_read_rate > 0.0 {
            let site = Box::leak(format!("store.read_log.stale_read.node[{}]", node_id).into_boxed_str());
            if self.ctx.rng(site).gen_bool(self.model.stale_read_rate) {
                tracing::warn!(%node_id, "Injecting stale read in read_log");
                // For stale reads, we could return an older version of data,
                // but for simplicity, we'll return None to simulate missing data
                return Ok(None);
            }
        }

        self.inner.read_log(idx)
    }

    fn kv_put(&mut self, k: bytes::Bytes, v: bytes::Bytes) -> Result<(), StoreError> {
        self.inner.kv_put(k, v)
    }

    fn kv_get(&mut self, k: &[u8]) -> Result<Option<bytes::Bytes>, StoreError> {
        self.inner.kv_get(k)
    }

    fn fsync(&mut self) -> Result<(), StoreError> {
        let node_id = self.ctx.node_id();
        let site = Box::leak(format!("store.fsync.node[{}]", node_id).into_boxed_str());
        if self.ctx.rng(site).gen_bool(self.model.fsync_fail_rate) {
            tracing::warn!(%node_id, "Injecting fsync failure");
            return Err(StoreError::FaultInjected);
        }
        self.inner.fsync()
    }
}
