<p align="center">
  <img src="logo.png" alt="Matchmaker Logo" width="320"/>
</p>

<p align="center">
  <a href="https://github.com/your-org/matchmaker-rs">
    <img src="https://img.shields.io/badge/language-Rust-orange?style=flat-square" alt="Rust">
    <img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="MIT License">
    <img src="https://img.shields.io/badge/build-passing-brightgreen?style=flat-square" alt="Build Status">
  </a>
</p>

---

# 🎮 Matchmaker RS

A **scalable matchmaking system** for video games, utilizing **WebSockets** for real-time state tracking and notifications.  
Designed for flexibility and performance, this project enables efficient player queue management and match creation.

---

## ✨ Features

- ⚡ **Real-time matchmaking** via WebSockets
- 🔄 Flexible queue and match management
- 🦀 Modular Rust backend
- 🌐 HTTP API for queue operations
- 📊 Benchmarking tools for performance analysis

---

## 🧩 Matchmakers

### 🏅 Elo

A matchmaker that pairs players based on their proximity in skill level. Uses an expanding range to pair players over time.  
**Max 2 teams**

**Settings**
```yaml
team_size:        # Number of players per team
scaling_factor:   # The rate at which the range expands over time (scaling_factor * time_in_queue in seconds)
max_skill_diff:   # Maximum allowable skill difference between players to be considered for a match
```

**Required metadata**
```yaml
elo:              # Numerical representation of player skill (e.g., Elo rating)
```

---

### 🧩 Flexible

A matchmaker that forms teams of variable sizes to reach a target team size, allowing for flexible group compositions.  
Supports multiple teams per match and can accommodate entries of different sizes within defined limits.

**Settings**
```yaml
target_team_size: # Desired number of players per team
min_entry_size:   # Minimum allowed size for an entry (group)
max_entry_size:   # Maximum allowed size for an entry (group)
num_teams:        # Number of teams to form per match
```

**Required metadata**

_None (entries are grouped by size; no specific player metadata required)._

---

## 📁 Project Structure

| Folder      | Description                                      |
|-------------|--------------------------------------------------|
| `common/`   | Shared Rust library code for matchmaking logic   |
| `http-api/` | Rust HTTP API server for queue/match operations  |
| `benchmark/`| Python benchmarking scripts and tools            |
| `bruno/`    | API testing scripts (Bruno)                      |

---

## 🚀 Prerequisites

- 🦀 Rust (latest stable, recommended via [rustup](https://rustup.rs/))
- 🐍 [uv](https://docs.astral.sh/uv/) (for Python dependency management)
- 🧪 (Optional) [Bruno](https://www.usebruno.com/) for API testing

---

## 🛠️ Build & Run (Rust Backend)

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

---

## 📈 Benchmarking

1. **Install Python dependencies:**
   ```sh
   cd benchmark
   uv sync
   ```

2. **Run benchmarks:**
   ```sh
   uv run main.py
   ```

---

## 🤝 Contributing

Contributions are welcome!  
Please open issues or pull requests for bug fixes, features, or improvements.

---

## 📄 License

This project is licensed under the MIT License.
