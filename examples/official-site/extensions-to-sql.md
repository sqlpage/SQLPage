## How SQLPage runs your SQL

SQLPage reads your SQL file and runs one statement at a time. For each statement, it

- decides whether to:
  - handle it inside SQLPage, or
  - prepare it as a (potentially slightly modified) sql statement on the database.
- extracts values from the request to pass them as prepared statements parameters
- runs [`sqlpage.*` functions](/functions)
- passes the database results to components

This page explains every step of the process,
with examples and details about differences between how SQLPage understands SQL and how your database does.

## What runs where

### Handled locally by SQLPage

- Static simple selects (a tiny, fast subset of SELECT)
- Simple variable assignments that use only literals or variables
 - All sqlpage functions
 

### Sent to your database

Everything else: joins, subqueries, arithmetic, database functions, `SELECT @@VERSION`, `CURRENT_TIMESTAMP`, `SELECT *`, expressions, `FROM`, `WHERE`, `GROUP BY`, `ORDER BY`, `LIMIT`/`FETCH`, `WITH`, `DISTINCT`, etc.

### Mixed statements using `sqlpage.*` functions

[`sqlpage.*` functions](/functions.sql) are executed by SQLPage; your database never sees them. They can run:

- Before the query, when used as values inside conditions or parameters.
- After the query, when used as top-level selected columns (applied per row).

Examples are shown below.

## Static simple selects

A *static simple select* is a very restricted `SELECT` that SQLPage can execute entirely by itself. This avoids back and forths between SQLPage and the database for trivial queries.

To be static and simple, a statement must satisfy all of the following:

- No `FROM`, `WHERE`, `GROUP BY`, `HAVING`, `ORDER BY`, `LIMIT`/`FETCH`, `WITH`, `DISTINCT`, `TOP`, windowing, locks, or other clauses.
- Each selected item is of the form `value AS alias`.
- Each `value` is either:
  - a literal (single-quoted string, number, boolean, or `NULL`), or
  - a variable (like `$name`, `:message`)

That’s it. If any part is more complex, it is not a static simple select and will be sent to the database.

#### Examples that ARE static (executed by SQLPage)

```sql
SELECT 'text' AS component, 'Hello' AS contents;
SELECT 'text' AS component, $name AS contents;
```

#### Examples that are NOT static (sent to the database)

```sql
-- Has string concatenation
select 'from' as component, 'handle_form.sql?id=' || $id as action;

-- Has WHERE
select 'text' as component, $alert_message as contents where $should_alert;

-- Uses database functions or expressions
SELECT 1 + 1 AS two;
SELECT CURRENT_TIMESTAMP AS now;
SELECT @@VERSION AS version; -- SQL Server variables
-- Uses a subquery
SELECT (select 1) AS one;
```

## Variables

SQLPage communicates information about incoming HTTP requests to your SQL code through prepared statement variables.

### Variable Types and Mutability

There are two types of variables in SQLPage:

1. **Request parameters** (immutable): URL parameters and form data from the HTTP request
2. **User-defined variables** (mutable): Variables created with the `SET` command

Request parameters cannot be modified after the request is received. This ensures the original request data remains intact throughout request processing.

### POST parameters

Form fields sent with POST are available as `:name`.

```sql
SELECT
  'form' AS component,
  'POST' AS method,
  'result.sql' AS action;

SELECT 'age' AS name, 'How old are you?' AS label, 'number' AS type;
```

```sql
-- result.sql
SELECT 'text' AS component, 'You are ' || :age || ' years old!' AS contents;
```

### URL parameters

Query-string parameters are available as `$name`.

```sql
SELECT 'text' AS component, 'You are ' || $age || ' years old!' AS contents;
-- /result.sql?age=42  →  You are 42 years old!
```

When a URL parameter is not set, its value is `NULL`.

### The SET command

`SET` creates or updates a user-defined variable in SQLPage (not in the database). Only strings and `NULL` are stored.

```sql
-- Give a default value to a variable
SET post_id = COALESCE($post_id, 0);

-- User-defined variables shadow URL parameters with the same name
SET my_var = 'custom value';  -- This value takes precedence over ?my_var=...
```

**Variable Lookup Precedence:**
- `$var`: checks user-defined variables first, then URL parameters
- `:var`: checks user-defined variables first, then POST parameters

This means `SET` variables always take precedence over request parameters when using `$var` or `:var` syntax.

**How SET works:**
- If the right-hand side is purely literals/variables, SQLPage computes it directly. See the section about *static simple select* above.
- If it needs the database (for example, calls a database function), SQLPage runs an internal `SELECT` to compute it and stores the first column of the first row of results.

Only a single textual value (**string or `NULL`**) is stored.
`SET id = 1` will store the string `'1'`, not the number `1`.

On databases with a strict type system, such as PostgreSQL, if you need a number, you will need to cast your variables: `SELECT * FROM post WHERE id = $id::int`.

Complex structures can be stored as json strings.

For larger temporary results, prefer temporary tables on your database; do not send them to SQLPage at all.

## `sqlpage.*` functions

Functions under the `sqlpage.` prefix run in SQLPage. See the [functions page](/functions.sql).

They can run:

### Before sending the query (as input values)

Used inside conditions or parameters, the function is evaluated first and its result is passed to the database.

```sql
SELECT *
FROM blog
WHERE slug = sqlpage.path();
```

### After receiving results (as top-level selected columns)

Used as top-level selected columns, the query is rewritten to first fetch the raw column, and the function is applied per row in SQLPage.

```sql
SELECT sqlpage.read_file_as_text(file_path) AS contents
FROM blog_posts;
```

## Performance

See the [performance page](/performance.sql) for details. In short:

- Statements sent to the database are prepared and cached.
- Variables and pre-computed values are bound as parameters.
- This keeps queries fast and repeatable.

## Working with larger temporary results

### Temporary tables in your database

When you reuse the same values multiple times in your page,
store them in a temporary table.

```sql
DROP TABLE IF EXISTS filtered_posts;
CREATE TEMPORARY TABLE filtered_posts AS
SELECT * FROM posts where category = $category;

select 'alert' as component, count(*) || 'results' as title
from filtered_posts;

select 'list' as component;
select name from filtered_posts;
```

### Small JSON values in variables

Useful for small datasets that you want to keep in memory.
See the [guide on JSON in SQL](/blog.sql?post=JSON+in+SQL%3A+A+Comprehensive+Guide).

```sql
set product = (
    select json_object('name', name, 'price', price)
    from products where id = $product_id
);
```

## CSV imports

When you write a compatible `COPY ... FROM 'field'` statement and upload a file with the matching form field name, SQLPage orchestrates the import:

- PostgreSQL: the file is streamed directly to the database using `COPY FROM STDIN`; the database performs the import.
- Other databases: SQLPage reads the CSV and inserts rows using a prepared `INSERT` statement. Options like delimiter, quote, header, escape, and a custom `NULL` string are supported. With a header row, column names are matched by name; otherwise, the order is used.

Example:

```sql
COPY my_table (col1, col2)
FROM 'my_csv'
(DELIMITER ';', HEADER);
```

The uploaded file should be provided in a form field with `'file' as type, 'my_csv' as name`.

## Data types

Each database has its own usually large set of data types.
SQLPage itself has a much more rudimentary type system.

### From the user to SQLPage

Form fields and URL parameters in HTTP are fundamentally untyped.
They are just sequences of bytes. SQLPage requires them to be valid utf8 strings.

SQLPage follows the convention that when a parameter name ends with `[]`, it represents an array.
Arrays in SQLPage are represented as JSON strings.

Example: In `users.sql?user[]=Tim&user[]=Tom`, `$user` becomes `'["Tim", "Tom"]'` (a JSON string exploitable with your database's builtin json functions).

### From SQLPage to the database

SQLPage sends only strings (`TEXT` or `VARCHAR`) and `NULL`s as parameters.

### From the database to SQLPage

Each row returned by the database becomes a JSON object
before its passed to components:

- Each column is a key. Duplicate column names turn into arrays.
- Numbers, booleans, text, and `NULL` map naturally.
- Dates/times become ISO strings.
- Binary data (BLOBs) becomes a data URL (with mime type auto-detection).

#### Example

```sql
SELECT
  1 AS one,
  'x' AS my_array, 'y' AS my_array,
  now() AS today,
  '<svg></svg>'::bytea AS my_image;
```

Produces something like:

```json
{
  "one": 1,
  "my_array": ["x", "y"],
  "today": "2025-08-30T06:40:13.894918+00:00",
  "my_image": "data:image/svg+xml;base64,PHN2Zz48L3N2Zz4="
}
```