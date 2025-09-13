INSERT INTO component(name, icon, introduced_in_version, description) VALUES 
('log', 'logs', '0.37.1', 'A component that writes messages to the server logs.
When a page runs, it prints your message to the terminal/console (standard error).
Use it to track what happens and troubleshoot issues.

### Where do the messages appear?

- Running from a terminal (Linux, macOS, or Windows PowerShell/Command Prompt): they show up in the window.
- Docker: run `docker logs <container_name>`.
- Linux service (systemd): run `journalctl -u sqlpage`.
- Output is written to [standard error (stderr)](https://en.wikipedia.org/wiki/Standard_streams#Standard_error_(stderr)).
');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'log', * FROM (VALUES
    -- top level
    ('message', 'The text to write to the server logs. It is printed when the page runs.', 'TEXT', TRUE, FALSE),
    ('level', 'How important the message is. One of ''trace'', ''debug'', ''info'' (default), ''warn'', ''error''. Not case-sensitive. Controls the level shown in the logs.', 'TEXT', TRUE, TRUE)
) x;

INSERT INTO example(component, description) VALUES
('log', '
### Record a simple message

This writes "Hello, World!" to the server logs.

```sql
SELECT ''log'' as component, ''Hello, World!'' as message;
```

Example output:

```text
[2025-09-13T22:30:14.722Z INFO  sqlpage::log from "x.sql" statement 1] Hello, World!
```

### Set the importance (level)

Choose how important the message is.

```sql
SELECT ''log'' as component, ''error'' as level, ''This is an error message'' as message;
```

Example output:

```text
[2025-09-13T22:30:14.722Z ERROR sqlpage::log from "x.sql" statement 2] This is an error message
```

### Log dynamic information

Include variables like a username.

```sql
set username = ''user''

select ''log'' as component,
        ''403 - failed for '' || coalesce($username, ''None'') as message,
        ''error'' as level;
```
')