//! # ftsim-engine::telemetry::tracing_layer
//!
//! A custom `tracing::Layer` that enriches log records with simulation-specific
//! context, such as the current simulation time, event ID, and node ID.

use super::{TelemetryBus, TracingContext};
use ftsim_types::id::NodeId;
use std::sync::{Arc, Mutex};
use tracing::{field::Field, span, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

pub struct SimContextLayer {
    context: Arc<Mutex<TracingContext>>,
}

impl SimContextLayer {
    pub fn new(bus: &TelemetryBus) -> Self {
        Self {
            context: bus.context(),
        }
    }
}

impl<S> Layer<S> for SimContextLayer
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).unwrap();
        let mut extensions = span.extensions_mut();

        // If the span has a `node_id` field, store it in the span's extensions.
        // This allows us to associate future log records within this span to the node.
        let mut visitor = NodeIdVisitor::default();
        attrs.record(&mut visitor);
        if let Some(node_id) = visitor.node_id {
            extensions.insert(NodeIdExtension(node_id));
        }
    }

    fn on_event(&self, event: &tracing::Event<'_>, ctx: Context<'_, S>) {
        let sim_ctx = self.context.lock().unwrap();

        // Find the node_id from the current span or its parents.
        let node_id = ctx.current_span().id().and_then(|id| {
            ctx.span_scope(id).and_then(|scope| {
                scope.from_root().find_map(|span| {
                    span.extensions().get::<NodeIdExtension>().map(|ext| ext.0)
                })
            })
        });

        // The actual injection of fields is handled by the formatting layer,
        // which can access this context. This layer's primary job is to
        // propagate the node_id via span extensions.
        // For direct field injection (less common), one would need a more
        // complex visitor setup.
    }
}

// --- Visitor helpers to extract and inject fields ---

#[derive(Default)]
struct NodeIdVisitor {
    node_id: Option<NodeId>,
}

impl tracing::field::Visit for NodeIdVisitor {
    fn record_u64(&mut self, field: &Field, value: u64) {
        if field.name() == "node_id" {
            self.node_id = Some(value as NodeId);
        }
    }
    fn record_i64(&mut self, _field: &Field, _value: i64) {}
    fn record_bool(&mut self, _field: &Field, _value: bool) {}
    fn record_str(&mut self, _field: &Field, _value: &str) {}
    fn record_error(
        &mut self,
        _field: &Field,
        _value: &(dyn std::error::Error + 'static),
    ) {
    }
    fn record_debug(&mut self, _field: &Field, _value: &dyn std::fmt::Debug) {}
}

struct NodeIdExtension(NodeId);
