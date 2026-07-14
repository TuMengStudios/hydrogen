# hydrogen

[中文](README_ZH.md)

Hydrogen is a Rust-powered data processing service for configuring, debugging, and running data cleaning jobs. It focuses on turning structured source data into cleaned, parsed output through task-based workflows.

## What It Does

- Manages data processing tasks from creation to execution and shutdown.
- Parses JSON-like source data with configurable field extraction, flattening, folding, ignore rules, defaults, and depth control.
- Provides debug tools for checking parser output before a task is started.
- Connects to Kafka sources and sinks for streaming data pipelines.
- Stores task metadata, logs, status, heartbeat, and processing counters in MySQL.
- Exposes HTTP endpoints for task management, parser debugging, Kafka checks, health checks, and runtime metrics.
- Restores tasks that were still marked as running when the service restarts.

## Project Shape

Hydrogen is built with Rust, Tokio, Axum, SQLx, Serde, and rdkafka. The main service coordinates HTTP APIs, database state, task execution, parser logic, logging, and connector utilities.
