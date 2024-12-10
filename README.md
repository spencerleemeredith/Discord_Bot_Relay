# Discord Bot Relaying Information

A real time Discord message monitoring and bridging system built with Rust. This bot creates a web dashboard that displays Discord messages and reactions in real time.

# Web Dashboard
This can be exchanged for part of your website. Use the Web Dashboard for testing purposes so you can see what it will look like on your website.

## Features

- Real time message bridging from Discord to web dashboard
- WebSocket based communication
- Discord themed web interface
- Emoji reaction support
- Automatic reconnection handling
- Concurrent execution using Tokio

## Architecture

The project consists of three main components:

1. **Discord Bot** (`src/main.rs`)
   - Built using Poise framework with Serenity
   - Monitors specific Discord channels
   - Handles message and reaction events
   - Forwards events to WebSocket server

2. **WebSocket Server** (`src/websocket_server.rs`)
   - Runs on `ws://localhost:8080`
   - Handles client connections
   - Broadcasts messages to connected clients
   - Manages connection state

3. **Web Dashboard** (`static/index.html`)
   - Runs on `http://localhost:8081`
   - Discord themed dark mode UI
   - Real time message updates
   - Displays user avatars and reactions
   - WebSocket client implementation

## Prerequisites

- Rust (latest stable version)
- A Discord Bot Token
- A Discord Channel ID for monitoring

## Setup

1. Clone the repository:
```bash
git clone https://github.com/yourusername/Discord_Bot_Relay.git
cd Discord_Bot_Relay
```

2. Create a `.env` file in the project root:
```env
DISCORD_TOKEN=your_discord_bot_token
BRIDGE_CHANNEL_ID=your_channel_id
WEBSOCKET_URL=ws://localhost:8080
```

3. Install dependencies:
```bash
cargo build
```

4. Run the bot:
```bash
cargo run
```

## Configuration

The bot uses the following environment variables:
- `DISCORD_TOKEN`: Your Discord bot token
- `BRIDGE_CHANNEL_ID`: The ID of the Discord channel to monitor
- `WEBSOCKET_URL`: WebSocket server URL (default: ws://localhost:8080)
- `RUST_LOG`: Log level (optional, defaults to "info")

## Dependencies

Key dependencies include:
- `tokio`: Async runtime
- `poise`: Discord bot framework
- `serenity`: Discord API client
- `actix-web`: Web server
- `tokio-tungstenite`: WebSocket implementation
- `serde`: Serialization framework

## Running the Project

1. For development:
```bash
# Debug build with better compile times and debug symbols
cargo run
```

2. For production:
```bash
# Release build with optimizations for better performance
cargo build --release
# Run the optimized binary
./target/release/project_quarm_dbot
```

The difference between debug and release builds:
- **Debug build** (`cargo build` or `cargo run`):
  - Faster compilation times
  - Includes debug information
  - No optimizations
  - Larger binary size
  - Better for development and debugging

- **Release build** (`cargo build --release`):
  - Slower compilation times
  - Includes all optimizations
  - Significantly faster runtime performance
  - Smaller binary size
  - Better for production deployment

3. Access the web dashboard:
   - Open `http://localhost:8081` in your web browser
   - Messages from the configured Discord channel will appear in real time
   - Reactions will be synchronized automatically

## Architecture Details

### Message Flow
1. Discord event occurs (message/reaction)
2. Bot captures event via Serenity
3. Event is serialized to JSON
4. WebSocket server broadcasts to connected clients
5. Web dashboard updates in real-time

### Concurrent Execution
- Uses Tokio for async runtime
- Separate tasks for:
  - Discord bot
  - WebSocket server
  - Web server

### Error Handling
- Graceful WebSocket reconnection
- Discord API error recovery
- Proper error propagation

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

MIT License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with Rust 🦀
- Powered by Serenity and Poise
- WebSocket implementation using tokio-tungstenite
- Web server using actix-web