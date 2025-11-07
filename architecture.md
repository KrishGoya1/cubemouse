Fantastic — this is exactly what seasoned engineers do before writing a single line of code: define **clear boundaries, flow, and reasoning** in an `ARCHITECTURE.md`.

Below is a production-grade draft for your project’s
`docs/ARCHITECTURE.md`.
It’s deeply technical, structured like internal architecture documentation used in professional engineering teams — focusing on modular design, async behavior, and maintainability.

---

```markdown
# 🧠 TrackPadLink — Architecture Documentation

**Version:** 1.0  
**Last Updated:** November 2025  
**Authors:** TrackPadLink Developers  
**Status:** Draft / Design Stage  

---

## 1. Overview

TrackPadLink is a **cross-device input streaming system** that allows a smartphone to act as a trackpad for a PC.  
The system is designed around **clear modular separation**, **binary protocols**, and **asynchronous networking** using Rust’s modern ecosystem.

This document describes the **architectural design**, **module boundaries**, and **data flow** within the system.

---

## 2. System Design Principles

> “Software should be modular enough to outgrow its first idea.”

TrackPadLink follows four core engineering principles:

1. **Isolation of Concerns** — Each module serves a single, defined purpose (networking, parsing, UI, etc.).
2. **Protocol First** — Communication structure drives design, not implementation convenience.
3. **Async by Default** — All I/O operations (networking, event handling) use non-blocking async primitives.
4. **Extensibility** — Future additions (gestures, encryption, GUIs) can be added without redesigning core modules.

---

## 3. High-Level Architecture

```

```
            ┌────────────────────────────┐
            │        Mobile Client       │
            │────────────────────────────│
            │  InputCapture   ───────┐   │
            │  DataEncoder    ───────┼──▶│
            │  ConnectionMgr  ───────┘   │
            │────────────────────────────│
            │  WebSocket Client           │
            └───────────▲────────────────┘
                        │ Binary (Opcodes)
                        ▼
            ┌────────────────────────────┐
            │          PC Server         │
            │────────────────────────────│
            │  ConnectionMgr  ───────────┐│
            │  ProtocolParser ───────────┼│
            │  InputTranslator───────────┘│
            │────────────────────────────│
            │  OS Cursor API             │
            └────────────────────────────┘
```

```

---

## 4. Rust Project Layout

### 🧩 Directory Structure

```

src/
├── main.rs
├── server/
│   ├── mod.rs              # Public exports
│   ├── connection.rs       # WebSocket listener & sessions
│   ├── protocol.rs         # Binary parsing & opcode dispatch
│   ├── translator.rs       # Converts to OS input events
│   └── handler.rs          # High-level event routing
├── utils/
│   ├── qr.rs               # QR generation for pairing
│   ├── logging.rs          # Centralized logging
│   └── timer.rs            # Keepalive & scheduling
└── config/
└── settings.rs         # Runtime configuration & flags

````

### Design Notes
- Each directory represents a **logical subsystem**.
- Each subsystem has its own `mod.rs` exporting only necessary items.
- Modules are decoupled via **traits** and **channels**, not direct calls.

---

## 5. Key Modules and Responsibilities

### 5.1 `server/connection.rs`
Handles all **WebSocket connections** between the PC and client device.

#### Responsibilities
- Start async listener (using `tokio-tungstenite`).
- Accept new connections.
- Spawn a new async task for each client.
- Forward raw frames to the protocol layer.

#### Exposed Traits
```rust
pub trait ConnectionHandler {
    async fn on_message(&mut self, bytes: Vec<u8>);
    async fn on_disconnect(&mut self);
}
````

#### Design Notes

* No packet interpretation inside this layer.
* Uses **MPSC channels** to feed bytes into the protocol parser.
* Clean separation ensures testability (fake client streams).

---

### 5.2 `server/protocol.rs`

Implements the **TrackPadLink Binary Protocol v1**.

#### Responsibilities

* Parse incoming binary packets.
* Validate `Opcode`, `Length`, and payload integrity.
* Dispatch to relevant handler via internal enum.

#### Core Types

```rust
enum Opcode {
    Move(i16, i16),
    Scroll(i16, i16),
    Click { button: u8, state: u8, fingers: u8 },
    KeepAlive(u32),
    Handshake { version: u8, device: u8, capabilities: u16 },
    Config { param: u8, value: Vec<u8> },
}
```

#### Design Notes

* Parsing is byte-offset based for speed.
* Malformed packets result in silent discard + optional debug log.
* Versioning allows forward compatibility.

---

### 5.3 `server/translator.rs`

Maps parsed events to **OS-level actions**.

#### Responsibilities

* Convert `Opcode::Move` → cursor movement.
* Convert `Opcode::Click` → mouse button event.
* Convert `Opcode::Scroll` → wheel delta.
* Use cross-platform crate (e.g., `enigo`, `mouse-rs`, or native bindings).

#### Design Goals

* OS abstraction via trait:

  ```rust
  trait InputDriver {
      fn move_cursor(&self, dx: i32, dy: i32);
      fn click(&self, button: MouseButton, state: ButtonState);
      fn scroll(&self, dx: i32, dy: i32);
  }
  ```
* Future drivers for Windows/macOS/Linux without rewriting logic.

---

### 5.4 `utils/qr.rs`

Generates a QR code containing the server’s IP + port in a standardized format.

#### Responsibilities

* Detect local network IP.
* Generate encoded string: `ws://<ip>:<port>`.
* Render QR code as ASCII for terminal display.

---

### 5.5 `utils/logging.rs`

Centralized structured logging.

#### Responsibilities

* Setup `env_logger` or `tracing` with consistent tags.
* Log connection events, protocol parsing, and runtime metrics.

Example Log:

```
[INFO] WS Connection established from 192.168.1.24
[DEBUG] Parsed MOVE: dx=12 dy=-3
[TRACE] Keepalive received (12500ms)
```

---

## 6. Concurrency & Runtime Design

### Runtime Model

* **Main Thread** spawns:

  * `WebSocket Listener Task` (awaits incoming connections)
  * `Input Event Task` (handles translation)
  * `Keepalive Task` (monitor)
* Each client runs on an independent async task with its own event queue.

### Communication Channels

| From       | To         | Mechanism                      | Purpose          |
| ---------- | ---------- | ------------------------------ | ---------------- |
| Connection | Protocol   | `tokio::mpsc::Sender<Vec<u8>>` | Frame forwarding |
| Protocol   | Translator | `tokio::mpsc::Sender<Event>`   | Decoded events   |
| Translator | OS         | Direct call                    | Action execution |

---

## 7. Error Handling & Resilience

| Type             | Strategy                                |
| ---------------- | --------------------------------------- |
| Connection Drop  | Auto-reconnect attempt on client side   |
| Malformed Packet | Log + discard silently                  |
| Input Failure    | Soft-fail (no crash), retry limited     |
| OS Event Failure | Log error, continue session             |
| Panic Recovery   | Top-level `tokio::spawn` error boundary |

All recoverable errors are logged with clear context tags (`[NET]`, `[PROTO]`, `[INPUT]`).

---

## 8. Configuration System

`config/settings.rs` exposes runtime configuration options via a typed struct:

```rust
struct AppConfig {
    ws_port: u16,
    sensitivity: f32,
    invert_scroll: bool,
    log_level: LogLevel,
}
```

* Default loaded from `config.toml`.
* Supports overrides via CLI flags or environment variables.
* Enables flexible runtime tuning without recompilation.

---

## 9. Testing Strategy

| Type                  | Description                                     |
| --------------------- | ----------------------------------------------- |
| **Unit Tests**        | For protocol parsing & encoding functions.      |
| **Integration Tests** | Simulated WebSocket client streams.             |
| **End-to-End Tests**  | Mock mobile client → PC translation validation. |
| **Benchmarks**        | Packet parsing performance (criterion.rs).      |

Example unit test target:

```rust
#[test]
fn test_move_packet_parsing() {
    let bytes = vec![0x01, 0x04, 0x05, 0x00, 0x02, 0x00];
    let event = parse_packet(bytes).unwrap();
    assert_eq!(event, Opcode::Move(5, 2));
}
```

---

## 10. Future Architectural Extensions

| Feature               | Description                | Architectural Impact                                |
| --------------------- | -------------------------- | --------------------------------------------------- |
| **Gestures**          | Pinch, swipe, rotate       | Add GestureHandler module                           |
| **Encryption**        | TLS or local symmetric key | Replace `tokio-tungstenite` with `tokio-native-tls` |
| **Config Sync**       | Device profiles            | Add config sync channel                             |
| **Desktop GUI**       | UI for QR and logs         | Integrate egui frontend                             |
| **Multi-client Mode** | Multiple trackpads         | Refactor server connection pool                     |

---

## 11. Deployment & Execution

### 11.1 PC Server

```bash
cargo run --release
```

→ Displays a QR code in terminal (e.g., `ws://192.168.1.50:9000`).

### 11.2 Mobile Client

Scan QR → connect → start sending binary frames.

---

## 12. Design Philosophy Summary

> **“The perfect small system is one where every byte, every module, and every decision earns its keep.”**

TrackPadLink’s architecture emphasizes:

* Predictable behavior under load.
* Separation of protocol and system logic.
* Simplicity and readability before micro-optimizations.
* Documentation as a *design deliverable*, not an afterthought.

---

**End of Document**

```

---

Would you like me to follow this with a short **`docs/PROTOCOL_SPEC.md`** file next — a dedicated standalone spec describing just the binary format (with byte offsets, bitfield breakdowns, and visual diagrams)?  
That file would complement this one and your `README.md` — making your repo look like a polished engineering-grade SDK project.
```
