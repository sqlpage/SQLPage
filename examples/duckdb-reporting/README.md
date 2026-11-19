# DuckDB CSV report

Run `docker compose up` from this folder and open [the monthly sales report](http://localhost:8080/).
Compare sales and revenue by month and region, then follow **View sales** to see
the transactions and the targets in effect on their sale dates.

Drop another monthly export into `sales/` with the same CSV columns to include
it in the report. Both pages query the typed CSV files directly with DuckDB's
`read_csv`. The summary groups dates into months with `time_bucket` and
aggregates with `GROUP BY ALL`. The detail page uses `arg_max_null` to pick
the regional target for the latest effective date on or before each sale.
SQLPage builds the drill-down links and binds their month and region to the
detail query. Request the detail page with `Accept: application/json` for JSON.

Run the end-to-end test from the repository root.

```sh
scripts/test-examples-hurl.sh examples/duckdb-reporting
```
