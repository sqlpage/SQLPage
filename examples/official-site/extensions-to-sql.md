# Extensions to SQL

SQLPage makes some special treatment before executing your SQL queries.

When executing your SQL file, SQLPage executes each query one at a time.
It doesn't send the whole file as-is to the database engine.

## Performance

See the [performance page](/performance.sql) for details on the optimizations
made to run your queries as fast as possible.

## Variables

SQL doesn't have its own mechanism for variables.
SQLPage implements variables in the following way:

### POST parameters

When sending a POST request, most often by sending a form with the
[form component](/component.sql?component=form), the form data is made
available as variables prefixed by a semi-colon.

So when this form is sent:

`form.sql`
```sql
SELECT
    'form' AS component,
    'POST' AS method, -- form defaults to using the HTTP POST method
    'result.sql' AS action;

SELECT
    'age' AS name,
    'How old are you?' AS label,
    'number' AS type;
```

It will make a request to this page:

`result.sql`
```sql
SELECT
    'text' AS component,
    'You are ' || :age || ' years old!' AS contents;
```

`:age` will be substituted by the actual value of the POST parameter.

### URL parameters

Likewise, URL parameters are available as variables prefixed by a dollar sign.

> URL parameters are often called GET parameters because they can originate
> from a form with 'GET' as the method.

So the previous example can be reworked to handle URL parameters:

`result.sql`
```sql
SELECT
    'text' AS component,
    'You are ' || $age || ' years old!' AS contents;
```

By querying this page with this URL: `/request.sql?age=42`
we would get `You are 42 years old!` as a response.

### The `SET` command

SQLPage overrides the behavior of `SET` statements in SQL to store variables in SQLPage itself instead of running the statement on the database. 

```sql
SET coalesced_post_id = COALESCE($post_id, 0);
```

`SET` statements are transformed into `SELECT` queries, and their result is stored in a `$`-variable:

```sql
SELECT COALESCE($post_id, 0);
```

We can override a previous `$`-variable:

```sql
SET post_id = COALESCE($post_id, 0);
```

### Limitations

`$`-variables and `:`-variables are stored by SQLPage, not in the database.

They can only store a string, or null.

As such, they're not designed to store table-valued results.
They will only store the first value of the first column:

```sql
CREATE TABLE t(a, b);
INSERT INTO t(a, b) VALUES (1, 2), (3, 4);

SET var = (SELECT * FROM t);

-- now $var contains '1'
```

Temporary table-valued results can be stored in two ways.

## Storing large datasets in the database with temporary tables

This is the most efficient method to store large values.
```sql
-- Database connections are reused and temporary tables are stored at the
-- connection level, so we make sure the table doesn't exist already
DROP TABLE IF EXISTS my_temp_table;
CREATE TEMPORARY TABLE my_temp_table AS
SELECT a, b
FROM my_stored_table ...

-- Insert data from direct values
INSERT INTO my_temp_table(a, b)
VALUES (1, 2), (3, 4);
```

## Storing rich structured data in memory using JSON

This can be more convenient, but should only be used for small values, because data
is copied from the database into SQLPage memory, and to the database again at each use.

You can use the [JSON functions from your database](/blog.sql?post=JSON+in+SQL%3A+A+Comprehensive+Guide).

Here are some examples with SQLite:
```sql
-- CREATE TABLE my_table(a, b);
-- INSERT INTO my_table(a, b)
-- VALUES (1, 2), (3, 4);

SET my_json = (
    SELECT json_group_array(a)
    FROM my_table
);
-- [1, 3]

SET my_json = json_array(1, 2, 3);
-- [1, 2, 3]
```

## Functions

Functions starting with `sqlpage.` are executed by SQLPage, not by your database engine.
See the [functions page](/functions.sql) for more details.

They're either executed before or after the query is run in the database.

### Executing functions *before* sending a query to the database

When they don't process results coming from the database:

```sql
SELECT * FROM blog WHERE slug = sqlpage.path()
```

`sqlpage.path()` will get replaced by the result of the function.

### Executing functions *after* receiving results from the database

When they process results coming from the database:

```sql
SELECT sqlpage.read_file_as_text(blog_post_file) AS title
FROM blog;
```

The query executed will be:

```sql
SELECT blog_post_file AS title FROM blog;
```

Then `sqlpage.read_file_as_text()` will be called on each row.

## Implementation details of variables and functions

All queries run by SQLPage in the database are first prepared, then executed.

Statements are prepared and cached the first time they're encountered by SQLPage.
Then those cached prepared statements are executed at each run, with parameter substitution.

All variables and function results are cast as text, to let the
database query optimizer know only strings (or nulls) will be passed.

Examples:

```sql
-- Source query
SELECT * FROM blog WHERE slug = sqlpage.path();

-- Prepared statement (SQLite syntax)
SELECT * FROM blog WHERE slug = CAST(?1 AS TEXT)
```

```sql
-- Source query
SET post_id = COALESCE($post_id, 0);

-- Prepared statement (SQLite syntax)
SELECT COALESCE(CAST(?1 AS TEXT), 0)
```
