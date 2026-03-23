# yam
## _Yet Another Monitor for Bitcoin Node_

yam is a terminal-based, fast, and lightweight monitor for your Bitcoin node and its surrounding services. Built with Rust and Ratatui, it provides a powerful dashboard and playground directly in your terminal.

- 📊 Real-time Bitcoin node monitoring
- 💻 System health tracking (CPU, RAM, Disk)
- 🛠️ In-app Bitcoin.conf editor
- ✨ RPC Playground with autocompletion

## Features

- **Dashboard**: Real-time view of blockchain info (blocks, difficulty), mempool status, and peer connections.
- **System Health**: Monitor your hardware stats including CPU load, memory usage, disk space, and temperatures.
- **Services Manager**: Check the status of integrated services such as Tor, Electrs, Mempool, and Explorer.
- **Config Editor**: Edit and save your `bitcoin.conf` file directly from the terminal UI.
- **RPC Playground**: A built-in terminal for running Bitcoin RPC commands with suggestions and history.

## Tech

yam uses a number of open source projects to work properly:

- [Rust] - The language of choice for performance and safety.
- [Ratatui] - A powerful TUI library for rich terminal interfaces.
- [Bitcoin Core RPC] - Direct interaction with your Bitcoin node.
- [Tokio] - Asynchronous runtime for high-performance I/O.
- [Sysinfo] - System and hardware monitoring.
- [Serde] - Serialization/deserialization for configuration files.

## Installation

yam requires [Rust](https://www.rust-lang.org/) (edition 2024+) to run.

Clone the repository and run the application using cargo.

```sh
cd yam
cargo run --release
```

For development (with debug logs):

```sh
cargo run
```

## Configuration

yam stores its configuration in `~/.yam/config.toml`. It also expects your Bitcoin configuration at `~/.bitcoin/bitcoin.conf` (configurable).

| Path | Description |
| ------ | ------ |
| `~/.yam/config.toml` | Application settings (RPC host, user, pass) |
| `~/.bitcoin/bitcoin.conf` | Bitcoin node configuration |

## Development

Want to contribute? Great!

yam is built with modularity in mind. Feel free to open a PR or suggest new features.

```sh
# Run tests
cargo test

# Build release binary
cargo build --release
```

[//]: # (These are reference links used in the body of this note and get stripped out when the markdown processor does its job.)

   [Rust]: <https://www.rust-lang.org/>
   [Ratatui]: <https://ratatui.rs/>
   [Bitcoin Core RPC]: <https://github.com/rust-bitcoin/rust-bitcoincore-rpc>
   [Tokio]: <https://tokio.rs/>
   [Sysinfo]: <https://github.com/GuillaumeGomez/sysinfo>
   [Serde]: <https://serde.rs/>
