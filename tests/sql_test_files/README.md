The sql files in this folder are all tested automatically. They are organized in
two subdirectories:

## `component_rendering/`

Files that depend on SQLPage's HTML rendering (components, shells, redirects,
etc.). Every file that does not start with `error_` must render a page that
contains the text "It works !" and no occurrence of the word "error" (case
insensitive). `error_` files should return a page containing the word "error"
and the rest of the file name. Files may include `nosqlite`, `nomssql`,
`nopostgres` or `nomysql` in their name to skip incompatible backends.

## `data/`

Files that only validate data-processing functions should live here. They must
return rows with an `actual` column plus either `expected` (exact match) or
`expected_contains` (substring match). Tests in this directory are fetched as
JSON and validated row by row.

### `data/database-specific/`

Files that only work on a single database engine (because they use
engine-specific SQL syntax) live in a subdirectory named after that database
(`sqlite`, `postgres`, `mysql`, `mssql`, `oracle`, `duckdb`, `snowflake`,
`generic`). They are run by a separate test, only when the current database
matches. Unlike the other directories, their file names do not need `_no...`
suffixes to exclude incompatible backends.