INSERT INTO blog_posts (title, description, icon, created_at, content)
VALUES
    (
        'Tracing SQLPage with OpenTelemetry and Grafana',
        'How to inspect requests, SQL queries, and database wait time with distributed tracing',
        'route-2',
        '2026-03-09',
        '
# Tracing SQLPage with OpenTelemetry and Grafana

When a page is slow, a log line telling you that the request took 1.8 seconds is only the start of the investigation. What you usually want to know next is where that time went:

- Did the request wait for a database connection?
- Which SQL file was executed?
- Which query took the longest?
- Did the delay start in SQLPage, in the reverse proxy, or in the database?

SQLPage now supports [OpenTelemetry](https://opentelemetry.io/), the standard way to emit distributed traces. Combined with Grafana, Tempo, Loki, and an OpenTelemetry collector, this gives you a detailed timeline of each request and lets you jump directly from logs to traces.

If you want a ready-to-run demo, see the [OpenTelemetry + Grafana example](https://github.com/sqlpage/SQLPage/tree/main/examples/opentelemetry-grafana), which this article is based on.

## What tracing gives you

With tracing enabled, one HTTP request becomes a tree of timed operations called spans. In a typical SQLPage app, you will see something like:

```text
[nginx] GET /todos
  └─ [sqlpage] GET /todos
       └─ [sqlpage] SQL website/todos.sql
            ├─ db.pool.acquire
            └─ db.query
```

This is immediately useful for:

- Debugging slow pages by seeing exactly which query consumed the time
- Detecting connection pool pressure by measuring time spent in `db.pool.acquire`
- Following one request end-to-end from the reverse proxy to SQLPage to PostgreSQL

Tracing is especially helpful in SQLPage because one request often maps cleanly to one SQL file. That makes traces easy to interpret even when you are not used to application performance tooling.

## The easiest way to try it

The simplest way to explore tracing is to run the example shipped with SQLPage:

```bash
cd examples/opentelemetry-grafana
docker compose up --build
```

That stack starts:

- nginx as the reverse proxy
- SQLPage
- PostgreSQL
- an OpenTelemetry collector
- Grafana Tempo for traces
- Grafana Loki for logs
- Promtail for log shipping
- Grafana for visualization

Then:

1. Open [http://localhost](http://localhost) and use the sample todo application.
2. Open [http://localhost:3000](http://localhost:3000) to access Grafana.
3. Inspect recent traces and logs from the default dashboard.
4. Open a trace to see the full span waterfall for a single request.

This setup is useful both as a demo and as a reference architecture for production deployments.

## Enabling tracing in SQLPage

Tracing is built into SQLPage. There is no plugin to install and no SQLPage-specific tracing configuration file to write.

You only need to set standard OpenTelemetry environment variables before starting SQLPage:

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4318"
export OTEL_SERVICE_NAME="sqlpage"
sqlpage
```

The `OTEL_EXPORTER_OTLP_ENDPOINT` variable tells SQLPage where to send traces.
The `OTEL_SERVICE_NAME` variable controls how the service appears in your tracing backend.

If `OTEL_EXPORTER_OTLP_ENDPOINT` is not set, SQLPage falls back to normal logging and tracing stays disabled.

## What you will see in the trace

The most useful spans emitted by SQLPage are:

- The HTTP request span, with attributes such as method, path, status code, and user agent
- The SQL file execution span, showing which `.sql` file handled the request
- The `db.pool.acquire` span, showing time spent waiting for a database connection
- The `db.query` span, containing the SQL statement and database system

In practice, that means you can answer questions like:

- Is the page slow because the SQL itself is slow?
- Is the request queued because the connection pool is exhausted?
- Is the delay happening before SQLPage even receives the request?

This is much more actionable than a single request duration number.

## Logs and traces together

Tracing is even more useful when logs and traces are connected.

In the example stack, SQLPage writes structured logs to stdout, Promtail forwards them to Loki, and Grafana lets you move from a log line to the matching trace using the trace id. This makes it possible to start from an error log and immediately inspect the full request timeline.

That workflow is often the difference between guessing and knowing.

## PostgreSQL correlation

SQLPage also propagates trace context to PostgreSQL through the connection `application_name`.
This makes it possible to correlate live PostgreSQL activity or database logs with the trace that triggered it.

For example, inspecting `pg_stat_activity` can show which trace is attached to a running query:

```sql
SELECT application_name, query, state
FROM pg_stat_activity;
```

If you also include `%a` in PostgreSQL''s `log_line_prefix`, your database logs can contain the same trace context.

## A practical debugging example

Suppose users report that a page occasionally becomes slow under load.

With tracing enabled, you might see that:

- the HTTP span is long
- the SQL file execution span is also long
- the `db.query` span is short
- but `db.pool.acquire` takes several hundred milliseconds

That immediately tells you the database query itself is not the problem. The real issue is contention on the connection pool. You can then increase `max_database_pool_connections`, reduce concurrent load, or review long-running requests that keep connections busy.

Without tracing, this kind of diagnosis usually requires guesswork.

## Deployment options

The example uses Grafana Tempo and Loki, but SQLPage is not tied to a single backend. Because it emits standard OTLP traces, you can also send data to:

- Jaeger
- Grafana Cloud
- Datadog
- Honeycomb
- New Relic
- Axiom

In small setups, SQLPage can often send traces directly to the backend. In larger deployments, placing an OpenTelemetry collector in the middle is usually better because it centralizes routing, batching, and authentication.

## When to enable tracing

Tracing is particularly valuable when:

- you are running SQLPage behind a reverse proxy
- several SQL files participate in user-facing workflows
- you want to understand production latency, not just local development behavior
- you need a shared debugging tool for developers and operators

If your application is already important enough to monitor, it is important enough to trace.

## Conclusion

SQLPage already makes the application logic easy to inspect because it lives in SQL files. Tracing extends that visibility to runtime behavior.

By enabling OpenTelemetry and connecting SQLPage to Grafana, you can see not just that a request was slow, but why it was slow, where the time was spent, and which query or resource caused the delay.

For a complete working setup, start with the [OpenTelemetry + Grafana example](https://github.com/sqlpage/SQLPage/tree/main/examples/opentelemetry-grafana) and adapt it to your own deployment.
'
    );
