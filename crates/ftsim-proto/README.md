# Crate: ftsim-proto

This crate provides the Software Development Kit (SDK) for implementing distributed protocols to be run within FTSim.

## Purpose

The primary goal of `ftsim-proto` is to define a clear, stable, and ergonomic API for protocol authors. It abstracts away the complexities of the simulation engine, providing a simple, event-driven interface.

## Core APIs

The crate exposes two key traits:

1.  **`Protocol<M>` (High-Level, Typed API):** This is the trait protocol authors should implement. It is generic over the protocol's message type `M`, which must be serializable. It provides a `Ctx<M>` object that offers a convenient, typed API for interacting with the simulator (e.g., `ctx.send(dest, &msg)`).

2.  **`ProtocolDyn` (Low-Level, Dynamic API):** This is the trait used internally by the simulation engine. It operates on raw byte slices (`&[u8]`) and is object-safe, allowing the engine to manage different protocol implementations via trait objects (`Box<dyn ProtocolDyn>`).

A blanket implementation is provided to automatically convert any `impl Protocol<M>` into a `Box<dyn ProtocolDyn>`, handling the serialization/deserialization layer transparently.

## Key Components

-   **`api.rs`:** Defines the `Protocol` and `ProtocolDyn` traits.
-   **`ctx_ext.rs`:** Defines the `Ctx<M>` context object, which provides the ergonomic, typed API for sending messages, setting timers, accessing storage, and interacting with the simulator's deterministic RNG.
-   **`protocols/`:** This directory contains example protocol implementations that demonstrate how to use the SDK.
    -   `primary_backup.rs`: A simple active replication protocol.
    -   `raft_lite/`: A partial implementation of the Raft consensus algorithm, focusing on leader election and log replication.

## Usage

To create a new protocol:
1.  Define your message `struct` or `enum` and derive `Serialize` and `Deserialize`. Use deterministic containers like `BTreeMap` or `IndexMap` instead of `HashMap`.
2.  Create a struct for your protocol's state.
3.  Implement `Protocol<YourMessageType>` for your state struct.
4.  In the `ftsim-cli` crate, register your protocol in the `REGISTRY` so the simulator can instantiate it.
