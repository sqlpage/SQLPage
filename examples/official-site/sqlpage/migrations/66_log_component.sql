INSERT INTO components(name, icon, description) VALUES 
('log', 'logs', 'A Component to log a message to the Servers STDOUT or Log file on page load')

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'log', * FROM (
    -- item level
    ('message', 'The message that needs to be logged', 'ANY', FALSE, FALSE),
    ('priority', 'The priority which the message should be logged with. Possible values are [''trace'', ''debug'', ''info'', ''warn'', ''error''] and are not case sensitive. If this value is missing or not matching any possible values, the default priority will be ''info''.', 'TEXT', FALSE, TRUE)
) x;

INSERT INTO example(component, description) VALUES
('log', '
### Hello World

Log a simple ''Hello, World!'' message on page load.

```sql
SELECT ''log'' as component,
        ''Hello, World!'' as message
```

Output example:

```
[2025-09-12T08:33:48.228Z INFO  sqlpage::log from file "index.sql" in statement 3] Hello, World!
```

### Priority

Change the priority to error.

```sql
SELECT ''log'' as component,
        ''This is a error message'' as message,
        ''error'' as priority
```

Output example:

```
[2025-09-12T08:33:48.228Z ERROR sqlpage::log from file "index.sql" in header] This is a error message
```

### Retrieve user data

```sql
set username =  ''user'' -- (retrieve username from somewhere)

select ''log''							  as component,
        ''403 - failed for '' || coalesce($username, ''None'') as output,
        ''error''							  as priority;
```

Output example:

```
[2025-09-12T08:33:48.228Z ERROR sqlpage::log from file "403.sql" in statement 7] 403 - failed for user
```
')