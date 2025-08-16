//! # ftsim-engine::scenario
//!
//! The scenario subsystem, responsible for loading scenario files and
//! scheduling the initial set of fault events.

use crate::{
    events::{Event, EventDiscriminant, FaultEventInternal, LinkModelChange},
    prelude::*,
    sim::Simulation,
};

/// Schedules a scenario's directives in the simulation.
pub fn load_and_schedule(sim: &mut Simulation, scenario: &Scenario) -> anyhow::Result<()> {
    let mut relative_time_base = 0;
    for directive in &scenario.directives {
        match directive {
            Directive::At(time, action) => {
                schedule(sim, *time, action.clone());
            }
            Directive::After { offset, action } => {
                relative_time_base += offset;
                schedule(sim, relative_time_base, action.clone());
            }
            Directive::Every {
                period,
                repeats,
                action,
            } => {
                for i in 0..*repeats {
                    let time = relative_time_base + (i as u128 * *period);
                    schedule(sim, time, action.clone());
                }
            }
        }
    }

    Ok(())
}

fn schedule(sim: &mut Simulation, when: SimTime, action: Action) {
    let ev = Event::Fault(action_to_internal(action));
    sim.schedule_at(when, ev, EventDiscriminant::fault());
}

fn action_to_internal(action: Action) -> FaultEventInternal {
    match action {
        Action::Crash { node, duration } => FaultEventInternal::Crash {
            node_id: node,
            duration,
        },
        Action::Restart { node } => FaultEventInternal::Restart { node_id: node },
        Action::Partition { sets } => FaultEventInternal::Partition { sets },
        Action::HealPartition => FaultEventInternal::HealPartition,
        Action::ClockSkew { node, skew } => FaultEventInternal::ClockSkew {
            node_id: node,
            skew_ns: skew,
        },
        Action::LinkDelay { link, dist } => FaultEventInternal::LinkModelUpdate {
            link_id: link,
            change: LinkModelChange::SetDelay(dist),
        },
        Action::LinkDrop { link, p } => FaultEventInternal::LinkModelUpdate {
            link_id: link,
            change: LinkModelChange::SetDrop(p),
        },
        Action::BroadcastBytes { payload_hex, proto_tag } => FaultEventInternal::BroadcastBytes {
            payload_hex,
            proto_tag,
        },
        Action::StoreFault { node, kind, rate } => FaultEventInternal::StoreFault {
            node_id: node,
            kind,
            rate,
        },
        Action::ByzantineFlip { node, enabled } => FaultEventInternal::ByzantineFlip {
            node_id: node,
            enabled,
        },
        Action::Custom { name, args } => FaultEventInternal::Custom { name, args },
    }
}
