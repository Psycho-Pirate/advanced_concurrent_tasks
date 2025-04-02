# Advanced Concurrent Tasks

## Overview
This Rust project is a library for managing concurrent task execution using the `tokio` runtime. It provides an `Orchestrator` to manage multiple `WorkerTask`s, handling success and failure scenarios, including critical and non-critical errors.

## Features
- Define and execute multiple tasks concurrently.
- Handle both critical and non-critical failures.
- Use async execution with `tokio`.
- Easily extendable and testable.

## Installation
Ensure you have Rust and Cargo installed. Then, add the library as a dependency:

```toml
[dependencies]
advanced_concurrent_tasks = { path = "./" }
```

## Usage

### Example
```rust
use advanced_concurrent_tasks::{Orchestrator, WorkerTask};
use std::sync::Arc;
use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    let mut orchestrator = Orchestrator::new();

    orchestrator.add_task(Arc::new(WorkerTask::new(1, 0.1, 4000, false)));
    orchestrator.add_task(Arc::new(WorkerTask::new(2, 1.0, 5500, true)));
    orchestrator.add_task(Arc::new(WorkerTask::new(3, 0.0, 7000, false)));
    
    orchestrator.run().await?;
    info!("All tasks completed");
    Ok(())
}
```

## Build and Run
```sh
cargo build
cargo test
```

## Running Tests
Run the test suite with:
```sh
cargo test
```

