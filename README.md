# LoadMaster Worker

Rust-powered worker for executing HTTP load tests.

## Tech Stack

- Rust
- Tokio (Async Runtime)
- Reqwest (HTTP Client)
- RabbitMQ (AMQP)
- Serde (JSON)

## Development

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build

# Run
cargo run
```

## Environment Variables

```env
RABBITMQ_URL=amqp://guest:guest@localhost:5672
RUST_LOG=info
```

## Docker

```bash
docker build -t loadmaster-worker .
docker run loadmaster-worker
```

## Performance

- Handles 10,000+ concurrent requests
- Zero-copy message passing
- Async I/O with Tokio
- Memory-safe (Rust guarantees)
