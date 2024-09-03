
INSERT INTO blog_posts (title, description, icon, created_at, content)
VALUES
    (
        'JSON in SQL: A Comprehensive Guide',
        'A comprehensive guide to working with JSON data in SQLite, PostgreSQL, MySQL, and SQL Server.',
        'braces',
        '2024-09-03',
        '
# JSON in SQL: A Comprehensive Guide

## Introduction

JSON (JavaScript Object Notation) is a popular data format for unstructured data. It allows storing composite data types, such as arrays and objects, in a single SQL value.
Many modern applications use JSON to store and exchange data. As a result, SQL databases have incorporated JSON support to allow developers to work with structured and semi-structured data within the same database.

This guide will cover JSON operations in SQLite, PostgreSQL, MySQL, and SQL Server, focusing on querying JSON data.

SQLPage uses JSON both to pass data to the database (when a SQLPage variable contains an array), and to pass data to components (when a component has a JSON parameter).
Thus, understanding how to work with JSON in SQL will allow you to fully leverage advanced SQLPage features.

JSON supports the following data types:

- **Objects**: A mapping between keys and values (`{ "key": "value" }`). Keys must be strings, and values can be of different types.
- **Arrays**: An ordered list of values enclosed in square brackets (`[ "value1", "value2" ]`). Values can be of different types.
- **Strings**: A sequence of characters enclosed in double quotes (`"Hello, World!"`).
- **Numbers**: An integer or floating-point number (`42`, `3.14`).
- **Boolean**: A true or false value (`true`, `false`).
- **Null**: A null value (`null`).

## Sample Table

We''ll use the following sample table for our examples:

```sql
CREATE TABLE users (
    id INT PRIMARY KEY,
    name VARCHAR(50),
    birthday DATE,
    group_name VARCHAR(50)
);

INSERT INTO users (id, name, birthday, group_name) VALUES
(1, ''Alice'', ''1990-01-15'', ''Admin''),
(2, ''Bob'', ''1985-05-22'', ''User''),
(3, ''Charlie'', ''1992-09-30'', ''User'');
```

## SQLite

SQLite provides increasingly better JSON support since version 3.38.0.
See [the list of JSON functions in SQLite](https://www.sqlite.org/json1.html) for more details.

### Creating a JSON object

We can use the standard `json_object()` function to create a JSON object from columns in a table:

```sql
SELECT json_object(''name'', name, ''birthday'', birthday) AS user_json
FROM users;
```

| user_json |
|-----------|
| {"name":"Alice","birthday":"1990-01-15"} |
| {"name":"Bob","birthday":"1985-05-22"} |
| {"name":"Charlie","birthday":"1992-09-30"} |

### Creating a JSON array

```sql
SELECT json_array(name, birthday, group_name) AS user_array
FROM users;
```

| user_array |
|------------|
| ["Alice","1990-01-15","Admin"] |
| ["Bob","1985-05-22","User"] |
| ["Charlie","1992-09-30","User"] |

### Aggregating multiple values into a JSON array

```sql
SELECT json_group_array(name) AS names
FROM users;
```

| names |
|-------|
| ["Alice","Bob","Charlie"] |

### Aggregating values into a JSON object

```sql
SELECT json_group_object(name, birthday) AS name_birthday_map
FROM users;
```

| name_birthday_map |
|-------------------|
| {"Alice":"1990-01-15","Bob":"1985-05-22","Charlie":"1992-09-30"} |


### Iterating over a JSON array

SQLite provides the `json_each()` table-valued function to iterate over JSON arrays. This function returns one row for each element in the JSON array.

```sql
SELECT value FROM json_each(''["Alice", "Bob", "Charlie"]'');
```

| value |
|-------|
| Alice |
| Bob |
| Charlie |

The `json_each()` function returns a table with several columns. The most commonly used are:

- `key`: The array index (0-based) for elements of a JSON array
- `value`: The value of the current element
- `type`: The type of the current element (e.g., ''text'', ''integer'', ''real'', ''true'', ''false'', ''null'')

For more complex JSON structures, you can use the `json_tree()` function, which recursively walks through the entire JSON structure.

These iteration functions can be used to test whether a value is present in a JSON array. For instance, to create a 
[multi-value select dropdown](documentation.sql?component=form#component) with pre-selected values, you can use the following query:

```sql
select json_group_array(json_object(
    ''label'', name
    ''value'', id,
    ''selected'', id in (select value from json_each_text($selected_ids))
)) as options
from users;
```

## PostgreSQL

PostgreSQL has extensive support for JSON, including the `jsonb` type, which offers better performance and more functionality than the `json` type.
See [the list of JSON functions in PostgreSQL](https://www.postgresql.org/docs/current/functions-json.html) for more details.

### Creating a JSON object

```sql
SELECT jsonb_build_object(''name'', name, ''birthday'', birthday) AS user_json FROM users;
```

| user_json |
|-----------|
| {"name": "Alice", "birthday": "1990-01-15"} |
| {"name": "Bob", "birthday": "1985-05-22"} |
| {"name": "Charlie", "birthday": "1992-09-30"} |

### Creating a JSON array

```sql
SELECT jsonb_build_array(name, birthday, group_name) AS user_array FROM users;
```

| user_array |
|------------|
| ["Alice", "1990-01-15", "Admin"] |
| ["Bob", "1985-05-22", "User"] |
| ["Charlie", "1992-09-30", "User"] |

### Aggregating multiple values into a JSON array

```sql
SELECT jsonb_agg(name) AS names FROM users;
```

| names |
|-------|
| ["Alice", "Bob", "Charlie"] |

### Aggregating values into a JSON object

```sql
SELECT jsonb_object_agg(name, birthday) AS name_birthday_map
FROM users;
```

| name_birthday_map |
|-------------------|
| `{"Alice": "1990-01-15", "Bob": "1985-05-22", "Charlie": "1992-09-30"}` |


### Iterating over a JSON array

```sql
SELECT name FROM jsonb_array_elements_text(''["Alice", "Bob", "Charlie"]''::jsonb) AS name;
```

| name |
|------|
| Alice |
| Bob |
| Charlie |

You can use this function to test whether a value is present in a JSON array. For instance, to create a
[multi-value select dropdown](documentation.sql?component=form#component) with pre-selected values, you can use the following query:

```sql
SELECT jsonb_agg(jsonb_build_object(
    ''label'', name,
    ''value'', id,
    ''selected'', id in (SELECT value FROM jsonb_array_elements_text($selected_ids::jsonb))
)) AS options
FROM users;
```

### Iterating over a JSON object

```sql
SELECT key, value
FROM jsonb_each_text(''{"name": "Alice", "birthday": "1990-01-15"}''::jsonb);
```

| key | value |
|-----|-------|
| name | Alice |
| birthday | 1990-01-15 |

### Querying JSON data

PostgreSQL allows you to query JSON data using the `->` and `->>` operators:

```sql
SELECT name, user_data->>''age'' AS age
FROM (
    SELECT name, jsonb_build_object(''age'', EXTRACT(YEAR FROM AGE(birthday))) AS user_data
    FROM users
) subquery
WHERE (user_data->>''age'')::int > 30;
```

| name | age |
|------|-----|
| Bob | 38 |

## MySQL / MariaDB

MySQL has good support for JSON operations starting from version 5.7.
See [the list of JSON functions in MySQL](https://dev.mysql.com/doc/refman/8.0/en/json-functions.html) for more details.

### Creating a JSON object

```sql
SELECT JSON_OBJECT(''name'', name, ''birthday'', birthday) AS user_json
FROM users;
```

| user_json |
|-----------|
| {"name": "Alice", "birthday": "1990-01-15"} |
| {"name": "Bob", "birthday": "1985-05-22"} |
| {"name": "Charlie", "birthday": "1992-09-30"} |

### Creating a JSON array

```sql
SELECT JSON_ARRAY(name, birthday, group_name) AS user_array
FROM users;
```

| user_array |
|------------|
| ["Alice", "1990-01-15", "Admin"] |
| ["Bob", "1985-05-22", "User"] |
| ["Charlie", "1992-09-30", "User"] |

### Aggregating multiple values into a JSON array

```sql
SELECT JSON_ARRAYAGG(name) AS names
FROM users;
```

| names |
|-------|
| ["Alice", "Bob", "Charlie"] |

### Aggregating values into a JSON object

```sql
SELECT JSON_OBJECTAGG(name, birthday) AS name_birthday_map
FROM users;
```

| name_birthday_map |
|-------------------|
| {"Alice": "1990-01-15", "Bob": "1985-05-22", "Charlie": "1992-09-30"} |

### Iterating over a JSON array

MySQL provides the JSON_TABLE() function to iterate over JSON arrays. This powerful function allows you to convert JSON data into a relational table format, making it easy to work with JSON arrays.

Here''s an example of how to use JSON_TABLE() to iterate over a JSON array:

```sql
SELECT jt.*
FROM JSON_TABLE(
  ''["Alice", "Bob", "Charlie"]'',
  ''$[*]'' COLUMNS(
    row_num FOR ORDINALITY,
    name VARCHAR(50) PATH ''$''
  )
) AS jt;
```

| row_num | name    |
|---------|---------|
| 1       | Alice   |
| 2       | Bob     |
| 3       | Charlie |

In this example:
- The first argument to JSON_TABLE() is the JSON array.
- `''$[*]''` is the path expression that selects all elements of the array.
- COLUMNS clause defines the structure of the output table:
  - `row_num FOR ORDINALITY` creates a column that numbers the rows.
  - `name VARCHAR(50) PATH ''$''` creates a column that contains the value of each array element.

You can also use JSON_TABLE() with more complex JSON structures:

```sql
SELECT jt.*
FROM JSON_TABLE(
  ''[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}, {"id": 3, "name": "Charlie"}]'',
  ''$[*]'' COLUMNS(
    row_num FOR ORDINALITY,
    id INT PATH ''$.id'',
    name VARCHAR(50) PATH ''$.name''
  )
) AS jt;
```

| row_num | id | name    |
|---------|------|---------|
| 1       | 1    | Alice   |
| 2       | 2    | Bob     |
| 3       | 3    | Charlie |

This approach allows you to easily iterate over JSON arrays and access their elements in a tabular format, which can be very useful for further processing or joining with other tables in your database.

### Iterating over a JSON object

MySQL provides the `JSON_TABLE` function to iterate over JSON objects:

```sql
SELECT jt.*
FROM JSON_TABLE(
  ''{"name": "Alice", "birthday": "1990-01-15"}'',
  ''$.*'' COLUMNS (
    value JSON PATH ''$''
  )
) AS jt;
```

| value |
|-------|
| "Alice" |
| "1990-01-15" |

### Querying JSON data

MySQL allows you to query JSON data using the `->` and `->>` operators:

```sql
SELECT name, user_data->''$.age'' AS age
FROM (
    SELECT name, JSON_OBJECT(''age'', YEAR(CURDATE()) - YEAR(birthday)) AS user_data
    FROM users
) subquery
WHERE user_data->''$.age'' > 30;
```

| name | age |
|------|-----|
| Bob | 38 |

## Microsoft SQL Server

SQL Server has support for JSON operations starting from SQL Server 2016.
See [the list of JSON functions in SQL Server](https://learn.microsoft.com/en-us/sql/t-sql/functions/json-functions-transact-sql?view=sql-server-ver16) for more details.

# JSON in SQL: A Comprehensive Guide

[Previous sections remain unchanged]

## Microsoft SQL Server

SQL Server has support for JSON operations starting from SQL Server 2016. It provides a comprehensive set of functions for working with JSON data.
See [the list of JSON functions in SQL Server](https://learn.microsoft.com/en-us/sql/t-sql/functions/json-functions-transact-sql?view=sql-server-ver16) for more details.

### Creating a JSON object

Use the `FOR JSON PATH` clause to create a JSON object:

```sql
SELECT (SELECT name, birthday FOR JSON PATH, WITHOUT_ARRAY_WRAPPER) AS user_json
FROM users;
```

| user_json |
|-----------|
| {"name":"Alice","birthday":"1990-01-15"} |
| {"name":"Bob","birthday":"1985-05-22"} |
| {"name":"Charlie","birthday":"1992-09-30"} |

Alternatively, you can use the `JSON_OBJECT` function:

```sql
SELECT JSON_OBJECT(''name'': name, ''birthday'': birthday) AS user_json
FROM users;
```

### Creating a JSON array

Use the `FOR JSON PATH` clause to create a JSON array:

```sql
SELECT (SELECT name, birthday, group_name FOR JSON PATH) AS user_array
FROM users;
```

| user_array |
|------------|
| [{"name":"Alice","birthday":"1990-01-15","group_name":"Admin"}] |
| [{"name":"Bob","birthday":"1985-05-22","group_name":"User"}] |
| [{"name":"Charlie","birthday":"1992-09-30","group_name":"User"}] |

You can also use the `JSON_ARRAY` function:

```sql
SELECT JSON_ARRAY(name, birthday, group_name) AS user_array
FROM users;
```

### Aggregating multiple values into a JSON array

Use the `FOR JSON PATH` clause to aggregate values into a JSON array:

```sql
SELECT (SELECT name FROM users FOR JSON PATH) AS names;
```

| names |
|-------|
| [{"name":"Alice"},{"name":"Bob"},{"name":"Charlie"}] |

Alternatively, use the `JSON_ARRAYAGG` function:

```sql
SELECT JSON_ARRAYAGG(name) AS names FROM users;
```

### Aggregating values into a JSON object

```sql
SELECT JSON_OBJECTAGG(name: birthday) AS name_birthday_map FROM users;
```

### Iterating over a JSON array

Use the `OPENJSON` function to iterate over JSON arrays:

```sql
SELECT value FROM OPENJSON(''["Alice", "Bob", "Charlie"]'');
```

| value |
|-------|
| Alice |
| Bob |
| Charlie |

### Iterating over a JSON object

Use `OPENJSON` to iterate over JSON objects:

```sql
SELECT *
FROM OPENJSON(''{"name": "Alice", "birthday": "1990-01-15"}'')
WITH (
    name NVARCHAR(50) ''$.name'',
    birthday DATE ''$.birthday''
);
```

| name | birthday |
|------|----------|
| Alice | 1990-01-15 |

### Querying JSON data

Use the `JSON_VALUE` function to extract scalar values from JSON:

```sql
SELECT JSON_VALUE(''{"age": 38}'', ''$.age'') AS age
```

| age |
|-----|
| 38 |

### Additional JSON Functions

SQL Server provides several other useful JSON functions:

- `ISJSON`: Tests whether a string contains valid JSON.
- `JSON_MODIFY`: Updates the value of a property in a JSON string.
- `JSON_PATH_EXISTS`: Tests whether a specified SQL/JSON path exists in the input JSON string.
- `JSON_QUERY`: Extracts an object or an array from a JSON string.

Example using `JSON_MODIFY`:

```sql
SELECT JSON_MODIFY(''{"name": "Alice", "age": 30}'', ''$.age'', 31) AS updated_json;
```

| updated_json |
|--------------|
| {"name": "Alice", "age": 31} |

This comprehensive guide covers the basics of working with JSON in SQLite, PostgreSQL, MySQL, and SQL Server. Each database has its own set of functions and syntax for JSON operations, but the general concepts remain similar across all platforms.
');