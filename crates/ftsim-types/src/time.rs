//! # ftsim-types::time
//!
//! Defines the representation of time within the simulation.
//! All simulation time is discrete and represented in nanoseconds to provide
//! high resolution for network and processing delays.

use crate::errors::SimError;
use serde::{Deserializer};

/// The fundamental unit of time in the simulation, measured in nanoseconds.
/// A `u128` provides an enormous range, preventing overflow for any practical simulation duration.
pub type SimTime = u128;

/// The start of simulation time.
pub const SIM_EPOCH: SimTime = 0;

/// The maximum representable simulation time.
pub const MAX_SIM_TIME: SimTime = u128::MAX;

/// Helper function to convert milliseconds to `SimTime`.
pub fn sim_from_ms(ms: u64) -> SimTime {
    (ms as u128) * 1_000_000
}

/// Helper function to convert microseconds to `SimTime`.
pub fn sim_from_us(us: u64) -> SimTime {
    (us as u128) * 1_000
}

/// Performs a checked addition on `SimTime`, returning an error on overflow.
pub fn checked_add(base: SimTime, offset: SimTime) -> Result<SimTime, SimError> {
    base.checked_add(offset)
        .ok_or(SimError::TimeOverflow { base, offset })
}

/// Performs a checked subtraction on `SimTime`, returning an error on overflow.
pub fn checked_sub(base: SimTime, offset: SimTime) -> Result<SimTime, SimError> {
    base.checked_sub(offset)
        .ok_or(SimError::TimeUnderflow { base, offset })
}

/// Custom deserializer for SimTime that handles both u64 and u128 values.
/// TOML only supports up to u64, so we need to handle the conversion manually.
pub fn deserialize_sim_time<'de, D>(deserializer: D) -> Result<SimTime, D::Error>
where
    D: Deserializer<'de>,
{
    struct SimTimeVisitor;

    impl<'de> serde::de::Visitor<'de> for SimTimeVisitor {
        type Value = SimTime;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a positive integer representing nanoseconds")
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value as u128)
        }

        fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if value < 0 {
                return Err(E::custom("SimTime cannot be negative"));
            }
            Ok(value as u128)
        }
    }

    deserializer.deserialize_any(SimTimeVisitor)
}

/// Custom deserializer for Option<SimTime>
pub fn deserialize_optional_sim_time<'de, D>(deserializer: D) -> Result<Option<SimTime>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptionalSimTimeVisitor;

    impl<'de> serde::de::Visitor<'de> for OptionalSimTimeVisitor {
        type Value = Option<SimTime>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an optional positive integer representing nanoseconds")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserialize_sim_time(deserializer).map(Some)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Some(value as u128))
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if value < 0 {
                return Err(E::custom("SimTime cannot be negative"));
            }
            Ok(Some(value as u128))
        }
    }

    deserializer.deserialize_option(OptionalSimTimeVisitor)
}
