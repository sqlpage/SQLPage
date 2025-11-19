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

Cookies can be set using the [cookie component](documentation.sql?component=cookie#component).

### Example

#### Set a cookie

Set a cookie called `username` to greet the user by name every time they visit the page:

```sql
select ''cookie'' as component, ''username'' as name, :username as value;

SELECT ''form'' as component;
SELECT ''username'' as name, ''text'' as type;
```

#### Read a cookie

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

If you need access to all headers at once, use [`sqlpage.headers()`](?function=headers) instead.
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
        '
Hashes a password with the Argon2id variant and outputs it in the [PHC string format](https://github.com/P-H-C/phc-string-format/blob/master/phc-sf-spec.md), ready to store in your users table.

Every call generates a brand new cryptographic salt so that two people choosing the same password still end up with different hashes, which defeats rainbow-table attacks and lets you safely reveal only the hash.

Use this function only when creating or resetting a password (for example while inserting a brand new user): it writes the stored value. Later, at login time, the [authentication component](documentation.sql?component=authentication#component) reads the stored hash, hashes the visitor''s password with the embedded salt and parameters, and grants access only if they match.

### Example

```sql
SELECT ''form'' AS component;
SELECT ''username'' AS name;
SELECT ''password'' AS name, ''password'' AS type;

INSERT INTO users (name, password_hash) VALUES (:username, sqlpage.hash_password(:password));
```

### Try online

You can try the password hashing function [on this page](/examples/hash_password.sql).
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
However, this can be changed by setting the `web_root` [configuration option](https://github.com/sqlpage/SQLPage/blob/main/configuration.md).
'
    );
INSERT INTO sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'environment_variable',
        '0.11.0',
        'variable',
        'Returns the value of the given [environment variable](https://en.wikipedia.org/wiki/Environment_variable).

### Example

```sql
SELECT ''text'' AS component;
SELECT ''The value of the HOME environment variable is '' AS contents;
SELECT sqlpage.environment_variable(''HOME'') as contents, true as code;
```'
    );
INSERT INTO sqlpage_function_parameters (
        "function",
        "index",
        "name",
        "description_md",
        "type"
    )
VALUES (
        'environment_variable',
        1,
        'name',
        'The name of the environment variable to read. Must be a literal string.',
        'TEXT'
    );
INSERT INTO sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'version',
        '0.11.0',
        'git-commit',
        'Returns the current version of SQLPage as a string.'
    );
INSERT INTO sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'exec',
        '0.12.0',
        'terminal-2',
        'Executes a shell command and returns its output as text.

### Example
    
#### Fetch data from a remote API using curl

```sql
select ''card'' as component;
select value->>''name'' as title, value->>''email'' as description
from json_each(sqlpage.exec(''curl'', ''https://jsonplaceholder.typicode.com/users''));
```

#### Notes

 - This function is disabled by default for security reasons. You can enable it by setting `"allow_exec" : true` in `sqlpage/sqlpage.json`. Enable it only if you trust all the users that can access your SQLPage server files (both locally and on the database).
 - Be careful when using this function, as it can be used to execute arbitrary shell commands on your server. Do not use it with untrusted input.
 - The command is executed in the current working directory of the SQLPage server process.
 - The command is executed with the same user as the SQLPage server process.
 - The environment variables of the SQLPage server process are passed to the command, including potentially sensitive variables such as `DATABASE_URL`.
 - The command is executed asynchronously, but the SQLPage server has to wait for it to finish before sending the result to the client.
   This means that the SQLPage server will not be blocked while the command is running, it will be able to serve other requests, but it will not be able to serve the current request until the command has finished.
   You should generally avoid long running commands.
 - If the program name is NULL, the result will be NULL.
 - If any argument is NULL, it will be passed to the command as an empty string.
 - If the command exits with a non-zero exit code, the function will raise an error.
 - Arbitrary SQL operations are not allowed as sqlpage function arguments. Use `SET` to assign the result of a SQL query to a variable, and then use that variable as an argument to `sqlpage.exec`.
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
        'exec',
        1,
        'program',
        'The name of the program to execute. Must be a literal string.',
        'TEXT'
    ),
    (
        'exec',
        2,
        'arguments...',
        'The arguments to pass to the program.',
        'TEXT'
    );
INSERT INTO sqlpage_functions (
        "name",
        "introduced_in_version",
        "icon",
        "description_md"
    )
VALUES (
        'url_encode',
        '0.12.0',
        'percentage',
        'Returns the given string, with all characters that are not allowed in a URL encoded.

### Example

```sql
select ''text'' as component;
select ''https://example.com/?q='' || sqlpage.url_encode($user_search) as contents;
```

#### Result

`https://example.com/?q=hello%20world`
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
        'url_encode',
        1,
        'string',
        'The string to encode.',
        'TEXT'
    );