CREATE TABLE IF NOT EXISTS sqlpage_functions (
    "name" TEXT PRIMARY KEY,
    "icon" TEXT,
    "description_md" TEXT,
    "return_type" TEXT
);
CREATE TABLE IF NOT EXISTS sqlpage_function_parameters (
    "function" TEXT REFERENCES sqlpage_functions("name"),
    "index" INTEGER,
    "name" TEXT,
    "description_md" TEXT,
    "type" TEXT
);
INSERT INTO sqlpage_functions ("name", "icon", "description_md")
VALUES (
        'cookie',
        'cookie',
        'Reads a [cookie](https://en.wikipedia.org/wiki/HTTP_cookie) with the given name from the request.
    Returns the value of the cookie as text, or NULL if the cookie is not present.

### Example

Read a cookie called `username` and greet the user by name:

```sql
SELECT ''text'' as component,
        ''Hello, '' || sqlpage.cookie(''username'') || ''!'' as contents;
```
'
    );
INSERT INTO sqlpage_function_parameters (
        "function",
        "index",
        "name",
        "description_md",
        "type"
    )
VALUES (
        'cookie',
        1,
        'name',
        'The name of the cookie to read.',
        'TEXT'
    );
INSERT INTO sqlpage_functions ("name", "icon", "description_md")
VALUES (
        'header',
        'heading',
        'Reads a [header](https://en.wikipedia.org/wiki/List_of_HTTP_header_fields) with the given name from the request.
    Returns the value of the header as text, or NULL if the header is not present.

### Example

Log the [`User-Agent`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/User-Agent) of the browser making the request in the database:

```sql
INSERT INTO user_agent_log (user_agent) VALUES (sqlpage.header(''user-agent''));
```
    '
    );
INSERT INTO sqlpage_function_parameters (
        "function",
        "index",
        "name",
        "description_md",
        "type"
    )
VALUES (
        'header',
        1,
        'name',
        'The name of the HTTP header to read.',
        'TEXT'
    );
INSERT INTO sqlpage_functions ("name", "icon", "description_md")
VALUES (
        'basic_auth_username',
        'user',
        'Returns the username from the [Basic Authentication](https://en.wikipedia.org/wiki/Basic_access_authentication) header of the request.
        If the header is not present, this function raises an authorization error that will prompt the user to enter their credentials.

### Example

```sql
SELECT ''authentication'' AS component,
    (SELECT password_hash from users where name = sqlpage.basic_auth_username()) AS password_hash,
    sqlpage.basic_auth_password() AS password;
```

'
    ),
    (
        'basic_auth_password',
        'key',
        'Returns the password from the [Basic Authentication](https://en.wikipedia.org/wiki/Basic_access_authentication) header of the request.
        If the header is not present, this function raises an authorization error that will prompt the user to enter their credentials.

### Example

```sql
SELECT ''authentication'' AS component,
    (SELECT password_hash from users where name = sqlpage.basic_auth_username()) AS password_hash,
    sqlpage.basic_auth_password() AS password;
```
'
    );