//! # ftsim-cli::logging
//!
//! Enhanced logging formatters for better visualization of simulation activity.

use std::fmt;
use tracing::{Event, Subscriber};
use tracing::field::Field;
use tracing_subscriber::{
    fmt::{format::Writer, FormatEvent, FormatFields},
    registry::LookupSpan,
};

/// A custom formatter that provides enhanced visualization for simulation events.
pub struct SimulationFormatter {
    timer: std::time::Instant,
}

impl SimulationFormatter {
    pub fn new() -> Self {
        Self {
            timer: std::time::Instant::now(),
        }
    }

    fn format_sim_time(sim_time_ns: u128) -> String {
        if sim_time_ns == 0 {
            "0ns".to_string()
        } else if sim_time_ns < 1_000 {
            format!("{}ns", sim_time_ns)
        } else if sim_time_ns < 1_000_000 {
            format!("{:.1}μs", sim_time_ns as f64 / 1_000.0)
        } else if sim_time_ns < 1_000_000_000 {
            format!("{:.1}ms", sim_time_ns as f64 / 1_000_000.0)
        } else {
            format!("{:.1}s", sim_time_ns as f64 / 1_000_000_000.0)
        }
    }

    fn extract_node_id(event: &Event) -> Option<u32> {
        let mut visitor = NodeIdExtractor::default();
        event.record(&mut visitor);
        visitor.node_id
    }
}

#[derive(Default)]
struct NodeIdExtractor {
    node_id: Option<u32>,
}

impl tracing::field::Visit for NodeIdExtractor {
    fn record_u64(&mut self, field: &Field, value: u64) {
        if field.name() == "node_id" {
            self.node_id = Some(value as u32);
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        if field.name() == "node_id" && value >= 0 {
            self.node_id = Some(value as u32);
        }
    }

    fn record_debug(&mut self, _field: &Field, _value: &dyn fmt::Debug) {}
    fn record_str(&mut self, _field: &Field, _value: &str) {}
    fn record_bool(&mut self, _field: &Field, _value: bool) {}
    fn record_error(&mut self, _field: &Field, _value: &(dyn std::error::Error + 'static)) {}
}

impl<S, N> FormatEvent<S, N> for SimulationFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let elapsed = self.timer.elapsed();
        let metadata = event.metadata();
        
        // Extract simulation time from span context if available
        let sim_time = if let Some(_span) = ctx.lookup_current() {
            // Try to extract sim_time from span extensions or fields
            // This is a simplified approach; in practice you might store this differently
            None
        } else {
            None
        };

        // Extract node ID from the event
        let node_id = Self::extract_node_id(event);

        // Format timestamp
        write!(writer, "\x1b[90m[{:>8.3}s]\x1b[0m ", elapsed.as_secs_f64())?;

        // Add simulation time if available
        if let Some(st) = sim_time {
            write!(writer, "\x1b[36m(sim: {})\x1b[0m ", Self::format_sim_time(st))?;
        }

        // Format level with color
        let level = metadata.level();
        let level_color = match *level {
            tracing::Level::ERROR => "\x1b[31m", // Red
            tracing::Level::WARN => "\x1b[33m",  // Yellow
            tracing::Level::INFO => "\x1b[32m",  // Green
            tracing::Level::DEBUG => "\x1b[34m", // Blue
            tracing::Level::TRACE => "\x1b[90m", // Gray
        };
        write!(writer, "{}[{:>5}]\x1b[0m ", level_color, level)?;

        // Add node ID if available
        if let Some(nid) = node_id {
            write!(writer, "\x1b[35m[N{}]\x1b[0m ", nid)?;
        }

        // Add target if it's not the default
        let target = metadata.target();
        if target != "events" && !target.starts_with(env!("CARGO_PKG_NAME")) {
            write!(writer, "\x1b[90m[{}]\x1b[0m ", target)?;
        }

        // Format the message
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

/// A simpler formatter for headless mode that emphasizes simulation events.
pub struct HeadlessFormatter;

impl<S, N> FormatEvent<S, N> for HeadlessFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let metadata = event.metadata();
        
        // Skip debug and trace messages in headless mode
        if matches!(*metadata.level(), tracing::Level::DEBUG | tracing::Level::TRACE) {
            return Ok(());
        }

        let target = metadata.target();
        let node_id = Self::extract_node_id(event);

        // Simplified format for key events
        match target {
            "events" => {
                // These are simulation engine events, give them special treatment
                if let Some(nid) = node_id {
                    write!(writer, "🎯 N{} ", nid)?;
                } else {
                    write!(writer, "🎯 SIM ")?;
                }
            }
            _ => {
                // Protocol or other events
                if let Some(nid) = node_id {
                    write!(writer, "📋 N{} ", nid)?;
                } else {
                    write!(writer, "📋 --- ")?;
                }
            }
        }

        // Format the message without extra metadata
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        
        writeln!(writer)
    }
}

impl HeadlessFormatter {
    fn extract_node_id(event: &Event) -> Option<u32> {
        let mut visitor = NodeIdExtractor::default();
        event.record(&mut visitor);
        visitor.node_id
    }
}
