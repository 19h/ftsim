//! # ftsim-engine::store::mem
//!
//! A simple, deterministic, in-memory storage implementation.
//! It uses `BTreeMap` to ensure that any iteration (not exposed in the API,
//! but good practice for determinism) is ordered.

use crate::prelude::*;
use bytes::Bytes;
use ftsim_proto::api::{LogIndex, LogRecord, StoreView as ProtoStoreView};
use std::collections::BTreeMap;

/// An in-memory key-value and log store.
#[derive(Default)]
pub struct MemStore {
    kv: BTreeMap<Bytes, Bytes>,
    log: Vec<LogRecord>,
}

impl MemStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Store for MemStore {
    fn as_view(&mut self) -> &mut dyn super::StoreView {
        self
    }
}

impl ProtoStoreView for MemStore {
    fn append_log(&mut self, rec: LogRecord) -> Result<LogIndex, StoreError> {
        let index = self.log.len() as LogIndex;
        self.log.push(rec);
        Ok(index)
    }

    fn read_log(&mut self, idx: LogIndex) -> Result<Option<LogRecord>, StoreError> {
        Ok(self.log.get(idx as usize).cloned())
    }

    fn kv_put(&mut self, k: Bytes, v: Bytes) -> Result<(), StoreError> {
        self.kv.insert(k, v);
        Ok(())
    }

    fn kv_get(&mut self, k: &[u8]) -> Result<Option<Bytes>, StoreError> {
        Ok(self.kv.get(k).cloned())
    }

    fn fsync(&mut self) -> Result<(), StoreError> {
        // In-memory store, fsync is a no-op.
        Ok(())
    }
}
