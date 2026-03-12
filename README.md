# Open Eyes

[![CI](https://github.com/mighty840/open-eyes/actions/workflows/ci.yml/badge.svg)](https://github.com/mighty840/open-eyes/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.84+-orange.svg)](https://www.rust-lang.org/)
[![Dioxus](https://img.shields.io/badge/Dioxus-0.7.3-purple.svg)](https://dioxuslabs.com/)
[![DuckDB](https://img.shields.io/badge/DuckDB-embedded-yellow.svg)](https://duckdb.org/)

**GenAI-powered open government data dashboard.** Pre-ingests German open government data (GovData.de, ~149K datasets) into DuckDB, then lets users ask natural language questions via a chat sidebar. An LLM generates SQL queries, executes them, and renders interactive ECharts visualizations — making open data truly accessible.

Built in Rust/Dioxus, starting with Germany, extensible to other EU CKAN portals.

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    User Browser (WASM)                     │
│  ┌─────────┐  ┌──────────────────┐  ┌─────────────────┐  │
│  │ Sidebar  │  │   Main Content   │  │   Chat Panel    │  │
│  │  Nav     │  │  (Home/Catalog/  │  │  NL Question →  │  │
│  │         │  │   Analytics)     │  │  Charts + Text  │  │
│  └─────────┘  └──────────────────┘  └─────────────────┘  │
└──────────────────────┬───────────────────────────────────┘
                       │ Server Functions
┌──────────────────────▼───────────────────────────────────┐
│                   Axum Server (SSR)                        │
│  ┌─────────────┐  ┌─────────────┐  ┌──────────────────┐  │
│  │  DuckDB     │  │  LLM Client │  │  CKAN Client     │  │
│  │  (embedded) │  │  (OpenAI    │  │  (pre-ingestion) │  │
│  │             │  │   compat.)  │  │                  │  │
│  └─────────────┘  └─────────────┘  └──────────────────┘  │
└───────────────────────────────────────────────────────────┘
```

## Workspace Crates

| Crate | Purpose |
|---|---|
| `open-eyes-core` | Config, models, DuckDB pool, LLM client, EChart option builders |
| `open-eyes-ingest` | CLI for CKAN crawling + resource download + DuckDB loading |
| `open-eyes-dashboard` | Dioxus fullstack app (web + server features) |

## Quick Start

### Prerequisites

- Rust 1.84+
- [Dioxus CLI](https://dioxuslabs.com/) (`cargo install dioxus-cli@0.7.3`)

### 1. Configure

Copy and edit the config file:

```bash
cp config.toml config.local.toml
# Edit config.local.toml with your LLM API key
```

Or use environment variables:

```bash
export OPEN_EYES_LLM_API_KEY="your-api-key"
export OPEN_EYES_LLM_BASE_URL="https://api.openai.com/v1"  # or any OpenAI-compatible endpoint
export OPEN_EYES_LLM_MODEL="gpt-4o-mini"
```

### 2. Ingest Data

```bash
# Crawl 10 datasets from GovData.de
cargo run -p open-eyes-ingest -- crawl --max-datasets 10

# Download and load CSV/JSON resources into DuckDB
cargo run -p open-eyes-ingest -- load
```

### 3. Run Dashboard

```bash
dx serve --package open-eyes-dashboard --platform web
```

Open http://localhost:8080 in your browser.

### 4. Ask Questions

Click the chat button (bottom-right) and ask questions like:
- "Welche Datensätze hat das Statistische Bundesamt?"
- "Show me the top 10 categories by dataset count"
- "What are the most common data formats?"

## Docker

```bash
# Build and start the dashboard
docker compose build
docker compose up dashboard

# Ingest data (one-off)
docker compose --profile ingest run ingest-crawl
docker compose --profile ingest run ingest-load
```

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `OPEN_EYES_LLM_API_KEY` | (empty) | LLM API key |
| `OPEN_EYES_LLM_BASE_URL` | `https://api.openai.com/v1` | OpenAI-compatible endpoint |
| `OPEN_EYES_LLM_MODEL` | `gpt-4o-mini` | Model to use |
| `OPEN_EYES_DUCKDB_PATH` | `./data/open-eyes.duckdb` | Database file path |
| `OPEN_EYES_PORT` | `8080` | Server port |
| `OPEN_EYES_CKAN_BASE_URL` | `https://ckan.govdata.de/api/3/action` | CKAN API endpoint |

## Development

```bash
# Check all crates
cargo build -p open-eyes-core
cargo build -p open-eyes-ingest
cargo build -p open-eyes-dashboard --features server

# Run tests
cargo test -p open-eyes-core
cargo test -p open-eyes-ingest

# Lint
cargo fmt --all
cargo clippy -p open-eyes-core -p open-eyes-ingest -- -D warnings
cargo clippy -p open-eyes-dashboard --features server -- -D warnings
```

## License

MIT
