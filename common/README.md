# Core Module - Matchmaker

The `core` module provides the foundational components for building a flexible and extensible matchmaking system in Rust. It is designed to support various matchmaking algorithms and queue management strategies, making it suitable for games, tournaments, or any scenario where entities need to be matched based on custom logic.

## Features
- **Queue Management:** Add, remove, and track teams or players in matchmaking queues.
- **Matchmaking Algorithms:** Plug in different matchmaking strategies by implementing the `Matchmaker` trait.
- **Extensible Entry Types:** Support for custom team or player types via the `QueueEntry` trait.
- **Unique Identification:** Uses UUIDs to uniquely identify teams or players in queues.

## Directory Structure
- `main.rs` - Entry point for the core module (if used as a binary).
- `matchmaker.rs` - Defines the `Matchmaker` trait and related logic.
- `queue_entry.rs` - Defines the `QueueEntry` trait for queueable entities.
- `queue.rs` - Implements the `Queue` struct for managing matchmaking queues.
- `queue_tracker.rs` - (Optional) Tracks queue state and statistics.
- `algo/` - Contains different matchmaking algorithm implementations (e.g., `flexible.rs`).

## Usage Example
```rust
use matchmaker::Queue;
use matchmaker::matchmaker::Matchmaker;
use matchmaker::queue_entry::QueueEntry;

// Define your own team/player type implementing QueueEntry
struct MyTeam { /* ... */ }
impl QueueEntry for MyTeam { /* ... */ }

// Implement a custom Matchmaker
struct MyMatchmaker;
impl Matchmaker<MyTeam> for MyMatchmaker { /* ... */ }

let matchmaker = Box::new(MyMatchmaker);
let mut queue = Queue::new("ranked_queue".to_string(), matchmaker);

// Add teams to the queue
queue.add_team(Box::new(MyTeam { /* ... */ }));

// Perform matchmaking
let matches = queue.matchmake().unwrap();
```

## Extending the Core Module
- **Add a new matchmaking algorithm:** Implement the `Matchmaker` trait and add your algorithm to the `algo/` directory.
- **Support new entry types:** Implement the `QueueEntry` trait for your custom team or player struct.

## Development
This module is part of a larger workspace. To build and test:

```sh
cd common
cargo build
cargo test
```

## License
MIT or Apache-2.0 (choose one and update as appropriate)

