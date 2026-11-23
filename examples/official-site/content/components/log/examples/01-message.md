### Record a simple message

This writes "Hello, World!" to the server logs.

```sql
SELECT 'log' as component, 'Hello, World!' as message;
```

Example output:

```text
[2025-09-13T22:30:14.722Z INFO  sqlpage::log from "x.sql" statement 1] Hello, World!
```

### Set the importance (level)

Choose how important the message is.

```sql
SELECT 'log' as component, 'error' as level, 'This is an error message' as message;
```

Example output:

```text
[2025-09-13T22:30:14.722Z ERROR sqlpage::log from "x.sql" statement 2] This is an error message
```

### Log dynamic information

Include variables like a username.

```sql
set username = 'user'

select 'log' as component,
        '403 - failed for ' || coalesce($username, 'None') as message,
        'error' as level;
```
