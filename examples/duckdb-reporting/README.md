# DuckDB CSV report

DuckDB reads `sales.csv` with a typed schema. SQLPage binds the `region` query parameter and returns matching sales as a JSON array ordered by sale ID.

Run `docker compose up` from this folder, then open [the South region report](http://localhost:8080/?region=South). The default region is North.

Run the end-to-end test from the repository root.

```sh
scripts/test-examples-hurl.sh examples/duckdb-reporting
```
