# Distributed Tracing and Logs for SQLPage with OpenTelemetry and Grafana

SQLPage has built-in support for [OpenTelemetry](https://opentelemetry.io/) (OTel),
an open standard for collecting traces, metrics, and logs from your applications.
When enabled, every HTTP request to SQLPage produces a **trace** — a timeline of
everything that happened to serve that request, from receiving it to querying the
database and rendering the response. SQLPage also emits structured request-aware
logs, which this example forwards to Grafana Loki so you can inspect logs and traces
side by side.

This is useful for:

- **Debugging slow pages**: see exactly which SQL query is taking the longest.
- **Diagnosing connection pool exhaustion**: see how long requests wait for a database connection.
- **End-to-end visibility**: follow a single user request from your reverse proxy (nginx, Caddy, etc.)
  through SQLPage and into PostgreSQL.

## Quick start (this example)

This directory contains a ready-to-run Docker Compose stack that demonstrates
the full tracing and logging pipeline. No prior OpenTelemetry experience is needed.

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/) and
  [Docker Compose](https://docs.docker.com/compose/install/) installed on your machine.

### Run

```bash
cd examples/opentelemetry-grafana
docker compose up --build
```

This starts eight services:

| Service          | Role                                                      | Port          |
|------------------|-----------------------------------------------------------|---------------|
| **nginx**        | Reverse proxy, creates the root trace span                | `localhost:80`  |
| **SQLPage**      | Your application, sends traces to the collector           | (internal 8080) |
| **PostgreSQL**   | Database                                                  | (internal 5432) |
| **OTel Collector** | Receives traces and forwards them to Tempo              | `localhost:4318` |
| **Tempo**        | Trace storage backend                                     | (internal 3200) |
| **Loki**         | Log storage backend                                       | (internal 3100) |
| **Promtail**     | Scrapes container logs and pushes them to Loki            | (internal) |
| **Grafana**      | Web UI to explore traces and logs                         | `localhost:3000` |

### Explore traces and logs

1. Open the todo app at [http://localhost](http://localhost) — add a few items, click to toggle them.
2. Open Grafana at [http://localhost:3000](http://localhost:3000).
3. The default home dashboard now shows recent traces and recent SQLPage logs.
4. Click any trace ID in the trace table to see the full span waterfall.
5. In the logs panel, click a `trace_id` derived field to jump straight to the matching trace.
6. In the left sidebar, click **Explore** (compass icon) if you want to search manually.
7. Select **Tempo** to search traces or **Loki** to search logs.

### What you will see in a trace

Each HTTP request produces a tree of **spans** (timed operations):

```
[nginx] GET /todos                         ← root span (created by nginx)
  └─ [sqlpage] GET /todos                  ← HTTP request span
       └─ [sqlpage] SQL website/todos.sql  ← SQL file execution
            ├─ db.pool.acquire             ← time waiting for a DB connection
            └─ db.query                    ← the actual SQL query
                 db.statement = "SELECT title, ..."
                 db.system = "PostgreSQL"
```

Key attributes on each span:

| Span               | Key attributes                                                |
|---------------------|--------------------------------------------------------------|
| HTTP request        | `http.method`, `http.target`, `http.status_code`, `http.user_agent` |
| SQL file execution  | `sqlpage.file` — which `.sql` file was executed              |
| `db.pool.acquire`   | `db.pool.size` — current pool size when acquiring            |
| `db.query`          | `db.statement` — the full SQL text; `db.system` — database type |

### What you will see in the logs

SQLPage writes one structured log line per event, for example:

```text
ts=2026-03-08T20:56:15.000Z level=info target=sqlpage::webserver::http msg="request completed" method=GET path=/ trace_id=4f2d...
```

Promtail scrapes these container logs, parses the logfmt fields, and sends them to Loki.
The homepage dashboard filters to the `sqlpage` container so you can see request logs update
live while you use the sample app.

### PostgreSQL correlation

SQLPage automatically sets the
[`application_name`](https://www.postgresql.org/docs/current/runtime-config-logging.html#GUC-APPLICATION-NAME)
on each database connection to include the W3C
[traceparent](https://www.w3.org/TR/trace-context/#traceparent-header).
This means you can:

- See trace IDs in `pg_stat_activity` when monitoring live queries:
  ```sql
  SELECT application_name, query, state FROM pg_stat_activity;
  -- application_name: sqlpage 00-abc123...-def456...-01
  ```
- Include trace IDs in PostgreSQL logs by adding `%a` to
  [`log_line_prefix`](https://www.postgresql.org/docs/current/runtime-config-logging.html#GUC-LOG-LINE-PREFIX).

### Testing pool pressure

To simulate database connection pool exhaustion (a common production issue),
reduce the pool size to 1 in `sqlpage/sqlpage.json`:

```json
{
    "listen_on": "0.0.0.0:8080",
    "max_database_pool_connections": 1
}
```

Restart (`docker compose restart sqlpage`), then open several browser tabs
to `http://localhost` simultaneously. In Grafana, you will see `db.pool.acquire`
spans with longer durations as requests queue up waiting for the single connection.

---

## How it works

### Enabling tracing in SQLPage

Tracing is **built into SQLPage** — there is nothing to install or compile.
It activates automatically when you set the `OTEL_EXPORTER_OTLP_ENDPOINT`
environment variable. When this variable is not set, SQLPage behaves exactly
as before (plain text logs, no tracing overhead).

**Minimal setup — just two environment variables:**

```bash
# Where to send traces (an OTLP-compatible endpoint)
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4318"

# A name to identify this service in traces
export OTEL_SERVICE_NAME="sqlpage"

# Now start SQLPage as usual
sqlpage
```

These are [standard OpenTelemetry environment variables](https://opentelemetry.io/docs/specs/otel/protocol/exporter/)
understood by all OTel-compatible tools. SQLPage reads them directly — no
`sqlpage.json` configuration is needed for tracing.

### The role of each component

**OpenTelemetry** is a standard, not a product. It defines a protocol (OTLP) for
sending trace data. Here is how the pieces fit together:

```
 Traces: SQLPage -> OTel Collector -> Tempo -> Grafana
 Logs:   SQLPage stdout/stderr -> Promtail -> Loki -> Grafana
```

- **SQLPage** generates trace data and sends it via the OTLP HTTP protocol.
- A **collector** (optional) receives traces and forwards them to one or more backends.
  Useful for buffering, sampling, or fanning out to multiple destinations.
  You can skip the collector and send directly from SQLPage to most backends.
- **Promtail** scrapes Docker container logs and forwards them to Loki.
- **Tempo** stores traces, **Loki** stores logs, and **Grafana** lets you search both.

### Trace context propagation

When a reverse proxy (like nginx) sits in front of SQLPage, you want the trace
to start at nginx and continue into SQLPage as a single, connected trace.
This works via the
[W3C Trace Context](https://www.w3.org/TR/trace-context/) standard:
nginx adds a `traceparent` HTTP header to the request it forwards to SQLPage,
and SQLPage reads it to continue the same trace.

Most modern reverse proxies and load balancers support this.
For nginx specifically, use the [`ngx_otel_module`](https://nginx.org/en/docs/ngx_otel_module.html)
(included in the `nginx:otel` Docker image).

---

## Setup guides by deployment scenario

### Self-hosted with Grafana Tempo and Loki

This is what the Docker Compose example in this directory uses.
[Grafana Tempo](https://grafana.com/oss/tempo/) is a free, open-source trace backend, and
[Grafana Loki](https://grafana.com/oss/loki/) is the corresponding log backend.

**Components:**
- [Grafana Tempo](https://grafana.com/docs/tempo/latest/) stores the traces.
- [Grafana Loki](https://grafana.com/docs/loki/latest/) stores the logs.
- [Grafana](https://grafana.com/docs/grafana/latest/) provides the web UI.
- An [OTel Collector](https://opentelemetry.io/docs/collector/) sits between
  SQLPage and Tempo (optional but recommended for production).
- [Promtail](https://grafana.com/docs/loki/latest/send-data/promtail/) scrapes container logs
  and forwards them to Loki.

**SQLPage environment variables:**

```bash
OTEL_EXPORTER_OTLP_ENDPOINT=http://<collector-or-tempo-host>:4318
OTEL_SERVICE_NAME=sqlpage
```

**Links:**
- [Tempo installation guide](https://grafana.com/docs/tempo/latest/setup/)
- [OTel Collector installation](https://opentelemetry.io/docs/collector/installation/)

### Self-hosted with Jaeger

[Jaeger](https://www.jaegertracing.io/) is another popular open-source tracing
backend. Version 2+ natively accepts OTLP — no collector needed.

**Start Jaeger with one command:**

```bash
docker run -d --name jaeger \
  -p 16686:16686 \
  -p 4317:4317 \
  -p 4318:4318 \
  jaegertracing/jaeger:latest
```

**SQLPage environment variables:**

```bash
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
OTEL_SERVICE_NAME=sqlpage
```

Open the Jaeger UI at [http://localhost:16686](http://localhost:16686) to explore traces.

**Links:**
- [Jaeger getting started](https://www.jaegertracing.io/docs/latest/getting-started/)

### Grafana Cloud

[Grafana Cloud](https://grafana.com/products/cloud/) has a free tier that
includes trace storage. SQLPage can send traces directly — no collector needed.

**SQLPage environment variables:**

```bash
OTEL_EXPORTER_OTLP_ENDPOINT=https://otlp-gateway-prod-<region>.grafana.net/otlp
OTEL_EXPORTER_OTLP_HEADERS="Authorization=Basic <base64-of-instance_id:api_token>"
OTEL_SERVICE_NAME=sqlpage
```

Replace:
- `<region>` with your Grafana Cloud region (e.g., `us-east-0`, `eu-west-2`).
  Find it in your Grafana Cloud portal under **My Account** > **Tempo**.
- `<base64-of-instance_id:api_token>` with the Base64 encoding of
  `<instance-id>:<cloud-api-token>`. Generate a token in your Grafana Cloud
  portal under **My Account** > **API Keys**.

  On macOS/Linux, generate the Base64 value with:
  ```bash
  echo -n "123456:glc_your_token_here" | base64
  ```

**Links:**
- [Send data via OTLP to Grafana Cloud](https://grafana.com/docs/grafana-cloud/send-data/otlp/send-data-otlp/)

### Datadog

[Datadog](https://www.datadoghq.com/) supports OTLP ingestion through the
Datadog Agent.

**1. Run the Datadog Agent** with OTLP ingest enabled:

```bash
docker run -d --name datadog-agent \
  -e DD_API_KEY=<your-datadog-api-key> \
  -e DD_OTLP_CONFIG_RECEIVER_PROTOCOLS_HTTP_ENDPOINT=0.0.0.0:4318 \
  -e DD_SITE=datadoghq.com \
  -p 4318:4318 \
  gcr.io/datadoghq/agent:latest
```

**2. Point SQLPage to the Agent:**

```bash
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
OTEL_SERVICE_NAME=sqlpage
```

Traces appear in the Datadog **APM > Traces** section.

**Links:**
- [OTLP ingestion in the Datadog Agent](https://docs.datadoghq.com/opentelemetry/setup/otlp_ingest_in_the_agent/)

### Honeycomb

[Honeycomb](https://www.honeycomb.io/) accepts OTLP directly — no collector needed.

**SQLPage environment variables:**

```bash
OTEL_EXPORTER_OTLP_ENDPOINT=https://api.honeycomb.io
OTEL_EXPORTER_OTLP_HEADERS="x-honeycomb-team=<your-api-key>"
OTEL_SERVICE_NAME=sqlpage
```

For the EU region, use `https://api.eu1.honeycomb.io` instead.

**Links:**
- [Send data with OpenTelemetry — Honeycomb docs](https://docs.honeycomb.io/send-data/opentelemetry/)

### New Relic

[New Relic](https://newrelic.com/) accepts OTLP directly.

**SQLPage environment variables:**

```bash
OTEL_EXPORTER_OTLP_ENDPOINT=https://otlp.nr-data.net
OTEL_EXPORTER_OTLP_HEADERS="api-key=<your-newrelic-license-key>"
OTEL_SERVICE_NAME=sqlpage
```

For the EU region, use `https://otlp.eu01.nr-data.net` instead.

Find your Ingest License Key in the New Relic UI under
**API Keys** (type: `INGEST - LICENSE`).

**Links:**
- [New Relic OTLP endpoint configuration](https://docs.newrelic.com/docs/opentelemetry/best-practices/opentelemetry-otlp/)

### Axiom

[Axiom](https://axiom.co/) accepts OTLP directly.

**SQLPage environment variables:**

```bash
OTEL_EXPORTER_OTLP_ENDPOINT=https://api.axiom.co
OTEL_EXPORTER_OTLP_HEADERS="Authorization=Bearer <your-api-token>,X-Axiom-Dataset=<your-dataset>"
OTEL_SERVICE_NAME=sqlpage
```

**Links:**
- [Send OpenTelemetry data to Axiom](https://axiom.co/docs/send-data/opentelemetry)

---

## Environment variable reference

These are [standard OpenTelemetry variables](https://opentelemetry.io/docs/specs/otel/protocol/exporter/),
not specific to SQLPage.

| Variable                          | Required? | Description                                    | Example                              |
|-----------------------------------|-----------|------------------------------------------------|--------------------------------------|
| `OTEL_EXPORTER_OTLP_ENDPOINT`    | Yes       | Base URL of the OTLP receiver                  | `http://localhost:4318`              |
| `OTEL_SERVICE_NAME`              | No        | Service name shown in traces (default: `unknown_service`) | `sqlpage`                 |
| `OTEL_EXPORTER_OTLP_HEADERS`    | No        | Comma-separated `key=value` pairs for auth headers | `api-key=abc123`                |
| `OTEL_EXPORTER_OTLP_PROTOCOL`   | No        | Protocol (default: `http/protobuf`)            | `http/protobuf`                      |
| `RUST_LOG`                        | No        | Filter which spans/logs are emitted            | `sqlpage=debug,tracing_actix_web=info` |

When `OTEL_EXPORTER_OTLP_ENDPOINT` is **not set**, SQLPage uses plain text
logging only (same behavior as versions before tracing support was added).

---

## Troubleshooting

### No traces appear

1. **Check that SQLPage sees the endpoint.** Look for this line in the startup logs:
   ```
   OpenTelemetry tracing enabled (OTEL_EXPORTER_OTLP_ENDPOINT is set)
   ```
   If you don't see it, the environment variable is not reaching SQLPage.

2. **Check that the collector/backend is reachable.** From the SQLPage host, try:
   ```bash
   curl -v http://<endpoint>:4318/v1/traces
   ```
   You should get a response (even if it's an error like "no data"), not a connection refused.

3. **Check the collector logs** for export errors (e.g., authentication failures).

### Traces are disconnected (nginx and SQLPage show as separate traces)

This means the `traceparent` header is not being propagated. Check that:

- Your reverse proxy is configured to inject/propagate the `traceparent` header.
- For nginx, you need the `ngx_otel_module` with `otel_trace_context propagate`
  in the location block. Setting `otel_span_name "$request_method $uri"` also keeps
  the nginx span name aligned with the actual request path. See the `nginx/nginx.conf`
  in this example.

### Spans are missing (e.g., no `db.query` spans)

The `RUST_LOG` / `OTEL_LOG_LEVEL` filter might be too restrictive.
SQLPage emits spans at the `INFO` level by default. Make sure your filter
includes `sqlpage=info`:

```bash
RUST_LOG="sqlpage=info,actix_web=info,tracing_actix_web=info"
```
