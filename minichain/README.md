# Minichain

A minimalist blockchain implementation in Rust that demonstrates core blockchain concepts including transactions, mining, and block creation.

## Features

- Simple blockchain structure with blocks and transactions
- Memory pool (mempool) for transaction queuing
- Gas-based transaction prioritization
- User management system with admin capabilities
- Async processing using Tokio
- Support for multiple nodes

## Getting Started

### Prerequisites

- Rust and Cargo (latest stable version)
- Environment variables set up in `.env` file

### Environment Setup

Create a `.env` file in the project root with the following configuration:

```env
BOOT_ADDR=127.0.0.1:8080
ADMIN_ADDR=0x4b6e99d0d66b1caa1f0377fbb7a97c12ecb3b457
MEMPOOL_BUF=10000
```

### Building

```bash
cargo build
```

### Running

```bash
cargo run
```

## Project Structure

- `src/`
  - `lib.rs` - Main library implementation and blockchain setup
  - `node.rs` - Node implementation, transaction processing, and blockchain logic
  - `main.rs` - Binary entry point
- `tests/` - Integration tests

## Core Components

### Blockchain

The blockchain consists of blocks that contain:
- Block number
- Timestamp
- Transactions

### Transactions

Transactions include:
- Unique ID
- Gas price (used for prioritization)
- Data payload
- Sender address

### Users

The system supports different user types:
- Admin users: Can fund other users and have special privileges
- Regular users: Can send transactions

### Mempool

The memory pool:
- Collects pending transactions
- Orders them by gas price
- Processes them in batches every 60 seconds

## Testing

Run the test suite with:

```bash
cargo test
```

For verbose output:

```bash
cargo test -- --nocapture
```

## License

[MIT License](LICENSE)