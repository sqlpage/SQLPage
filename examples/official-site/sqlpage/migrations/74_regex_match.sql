INSERT INTO
    sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES
    (
        'regex_match',
        '0.43.0',
        'regex',
        'Matches a text value against a regular expression and returns the capture groups as a JSON object.

If the text matches the pattern, the result contains one entry for each capture group that matched:
- key `0` contains the full match
- named groups like `(?<name>...)` use their name as the JSON key
- unnamed groups like `( ... )` use their numeric index as a string

If the text does not match, this function returns `NULL`.

### Example: custom routing from `404.sql`

This function is especially useful in a custom [`404.sql` page](/your-first-sql-website/custom_urls.sql),
where you want to turn a dynamic URL into variables your SQL can use.

For example, suppose you want `/categories/{category}/post/{id}` URLs such as `/categories/sql/post/42`,
but there is no physical `categories/sql/post/42.sql` file on disk.
You can put a `categories/404.sql` file in your project and extract the dynamic parts from the URL:

#### `categories/404.sql`
```sql
set route = sqlpage.regex_match(
  ''/categories/(?<category>\w+)/post/(?<id>\d+)'',
  sqlpage.path()
);

select ''redirect'' as component, ''/404'' as link
where $route is null;

select ''text'' as component;
select
  ''Category: '' || ($route->>''category'') || '' | Post id: '' || ($route->>''id'') as contents;
```

If the current path is `/categories/sql/post/42`, `sqlpage.regex_match()` returns:

```json
{"0":"/categories/sql/post/42","category":"sql","id":"42"}
```

You can then use those extracted values to query your database:

```sql
select title, body
from posts
where category = $route->>''category''
  and id = cast($route->>''id'' as integer);
```

### Details

- Quick regex reminder:
  - `\w+` matches one or more "word" characters
  - `\d+` matches one or more digits
  - `(?<name>...)` creates a named capture group
- Some databases, such as MySQL and MariaDB, treat backslashes specially inside SQL strings.
  In those databases, you may need to write `\\w` and `\\d`, or use portable character classes such as `[A-Za-z0-9_]` and `[0-9]` instead.
- In SQLite, PostgreSQL, and some other databases, you can read fields from the returned JSON with `->` and `->>`
- On databases that do not support that syntax, use their JSON extraction function instead, such as `json_extract($route, ''$.category'')`
- For the full regular expression syntax supported by SQLPage, see the Rust `regex` crate documentation:
  [regex syntax reference](https://docs.rs/regex/latest/regex/#syntax)
- If the input text is `NULL`, the function returns `NULL`
- If an optional capture group does not match, that key is omitted from the JSON object
- If the regular expression is invalid, SQLPage returns an error

The returned JSON can then be processed with your database''s JSON functions.
'
    );

INSERT INTO
    sqlpage_function_parameters (
        "function",
        "index",
        "name",
        "description_md",
        "type"
    )
VALUES
    (
        'regex_match',
        1,
        'pattern',
        'The regular expression pattern to match against the input text. Named capture groups such as `(?<name>...)` are supported.',
        'TEXT'
    ),
    (
        'regex_match',
        2,
        'text',
        'The text to match against the regular expression. Returns `NULL` when this argument is `NULL`.',
        'TEXT'
    );
