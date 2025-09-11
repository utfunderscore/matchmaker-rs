<p align="center">
  <img src="logo.png" alt="Matchmaker Logo" width="500"/>
</p>

A scalable matchmaking system for video games, utilizing WebSockets for real-time state tracking and notifications. Designed for flexibility and performance, this project enables efficient player queue management and match creation.

## Features

- Real-time matchmaking via WebSockets
- Flexible queue and match management
- Modular Rust backend
- HTTP API for queue operations
- Benchmarking tools for performance analysis

## Matchmakers
### Elo
A matchmaker that pairs players based on their proximity in skill level. Uses an expanding range to
pair players over time. Max 2 teams

**Settings**:
- `team_size`: Number of players per team.
- `scaling_factor`: Determines the max and min skill range for matching `scaling_factor * time_in_queue (in seconds)`.
- `max_skill_diff`: Maximum allowable skill difference between players to be considered for a match.

**Required metadata**
- `elo`: Numerical representation of player skill (e.g., Elo rating).


## Project Structure

- `common/` – Shared Rust library code for matchmaking logic
- `http-api/` – Rust HTTP API server for queue and match operations
- `benchmark/` – Python benchmarking scripts and tools
- `bruno/` – API testing scripts (Bruno)

## Prerequisites

- Rust (latest stable, recommended via [rustup](https://rustup.rs/))
- [uv](https://docs.astral.sh/uv/) (for Python dependency management)
- (Optional) [Bruno](https://www.usebruno.com/) for API testing

## Build & Run (Rust Backend)

1. **Clone the repository:**
   ```sh
   git clone <repo-url>
   cd matchmaker
   ```

2. **Build the Rust workspace:**
   ```sh
   cargo build --release
   ```

3. **Run the HTTP API server:**
   ```sh
   cd http-api
   cargo run --release
   ```

   The server will start and listen for HTTP/WebSocket connections.

## Benchmarking

1. **Install Python dependencies:**
   ```sh
   cd benchmark
   uv sync
   ```

2. **Run benchmarks:**
   ```sh
   uv run main.py
   ```

## Contributing

Contributions are welcome! Please open issues or pull requests for bug fixes, features, or improvements.

## License

This project is licensed under the MIT License.

