# SQLPage + OpenTelemetry + Grafana Example

This example demonstrates end-to-end distributed tracing with SQLPage.

## Architecture

```
Browser → nginx (with OTel module) → SQLPage → PostgreSQL
                  ↓                      ↓
             OTel Collector  ←───────────┘
                  ↓
             Grafana Tempo
                  ↓
              Grafana (UI)
```

## How to run

```bash
docker compose up --build
```

## How to explore traces

1. Open the todo app at [http://localhost](http://localhost) and add a few items.
2. Open Grafana at [http://localhost:3000](http://localhost:3000).
3. Go to **Explore** → select the **Tempo** datasource.
4. Search for traces by service name (`nginx` or `sqlpage`).

## What to look for in traces

Each HTTP request produces a trace with:

- **nginx root span** — the entry point, with `traceparent` propagated to SQLPage
- **SQLPage HTTP request span** — child of nginx, created by `tracing-actix-web`
- **`sqlpage.exec`** — SQL file execution, with `sqlpage.file` attribute
- **`db.pool.acquire`** — time spent waiting for a database connection from the pool
- **`db.query`** — individual SQL statement execution, with `db.statement` and `db.system`

## Testing pool pressure

To see `db.pool.acquire` spans with queuing time, reduce the pool size:

```json
// sqlpage/sqlpage.json
{
    "listen_on": "0.0.0.0:8080",
    "max_database_pool_connections": 1
}
```

Then send concurrent requests (e.g., open multiple browser tabs quickly) and observe the acquire duration in Grafana.
