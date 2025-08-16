//! # ftsim-engine::rng
//!
//! Defines the discipline for using the master Random Number Generator.
//! The `RngDiscipline` wrapper ensures that every use of the RNG is
//! associated with a site label and recorded for auditing.

use rand::RngCore;
use rand_chacha::ChaCha20Rng;
use std::collections::BTreeMap;

/// A wrapper around the master RNG to enforce recording of its usage.
pub struct RngDiscipline<'a> {
    rng: &'a mut ChaCha20Rng,
    recorder: &'a mut Recorder,
    site_label: &'static str,
}

impl<'a> RngDiscipline<'a> {
    pub fn new(
        rng: &'a mut ChaCha20Rng,
        recorder: &'a mut Recorder,
        site_label: &'static str,
    ) -> Self {
        Self {
            rng,
            recorder,
            site_label,
        }
    }
}

/// Delegate the `RngCore` trait to the inner RNG, but record each call.
impl<'a> RngCore for RngDiscipline<'a> {
    fn next_u32(&mut self) -> u32 {
        self.recorder.record_draw(self.site_label);
        self.rng.next_u32()
    }
    fn next_u64(&mut self) -> u64 {
        self.recorder.record_draw(self.site_label);
        self.rng.next_u64()
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.recorder.record_draw(self.site_label);
        self.rng.fill_bytes(dest)
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.recorder.record_draw(self.site_label);
        self.rng.try_fill_bytes(dest)
    }
}

/// Records all deterministic decisions made during a simulation.
pub struct Recorder {
    seed: u64,
    rng_sites: BTreeMap<&'static str, u64>,
}

impl Recorder {
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            rng_sites: BTreeMap::new(),
        }
    }

    /// Records that a random number was drawn at a specific site.
    pub fn record_draw(&mut self, site_label: &'static str) {
        *self.rng_sites.entry(site_label).or_insert(0) += 1;
    }
}
