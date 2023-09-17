CREATE TABLE IF NOT EXISTS sqlpage_functions (
    "name" TEXT PRIMARY KEY,
    "icon" TEXT,
    "description_md" TEXT,
    "return_type" TEXT,
    "introduced_in_version" TEXT
);
CREATE TABLE IF NOT EXISTS sqlpage_function_parameters (
    "function" TEXT REFERENCES sqlpage_functions("name"),
    "index" INTEGER,
    "name" TEXT,
    "description_md" TEXT,
    "type" TEXT
);
INSERT INTO sqlpage_functions (
        "name",
        "return_type",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'cookie',
        'TEXT',
        '0.7.1',
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
INSERT INTO sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'header',
        '0.7.2',
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
INSERT INTO sqlpage_functions (
        "name",
        "return_type",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'basic_auth_username',
        'TEXT',
        '0.7.2',
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
        'TEXT',
        '0.7.2',
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
INSERT INTO sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'hash_password',
        '0.7.2',
        'spy',
        'Hashes a password using the [Argon2](https://en.wikipedia.org/wiki/Argon2) algorithm.
    The resulting hash can be stored in the database and then used with the [authentication component](documentation.sql?component=authentication#component).

### Example

```sql
SELECT ''form'' AS component;
SELECT ''username'' AS name;
SELECT ''password'' AS name, ''password'' AS type;

INSERT INTO users (name, password_hash) VALUES (:username, sqlpage.hash_password(:password));
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
        'hash_password',
        1,
        'password',
        'The password to hash.',
        'TEXT'
    );
INSERT INTO sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'random_string',
        '0.7.2',
        'arrows-shuffle',
        'Returns a cryptographically secure random string of the given length.

### Example

Generate a random string of 32 characters and use it as a session ID stored in a cookie:

```sql
INSERT INTO login_session (session_token, username) VALUES (sqlpage.random_string(32), :username)
RETURNING 
    ''cookie'' AS component,
    ''session_id'' AS name,
    session_token AS value;
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
        'random_string',
        1,
        'length',
        'The length of the string to generate.',
        'INTEGER'
    );
INSERT INTO sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'current_working_directory',
        '0.11.0',
        'folder-question',
        'Returns the [current working directory](https://en.wikipedia.org/wiki/Working_directory) of the SQLPage server process.

### Example

```sql
SELECT ''text'' AS component;
SELECT ''Currently running from '' AS contents;
SELECT sqlpage.current_working_directory() as contents, true as code;
```

#### Result

Currently running from `/home/user/my_sqlpage_website`

#### Notes

The current working directory is the directory from which the SQLPage server process was started.
By default, this is also the directory from which `.sql` files are loaded and served.
However, this can be changed by setting the `web_root` [configuration option](https://github.com/lovasoa/SQLpage/blob/main/configuration.md).
');