---
icon: terminal-2
since: 0.12.0
category: server-and-runtime
summary: Run a program on the server and capture its output.
parameters:
  - name: program
    type: TEXT
    required: true
    description: The name of the program to execute. Must be a literal string.
  - name: arguments...
    type: TEXT
    description: The arguments to pass to the program.
---

Executes a shell command and returns its output as text.

### Example

#### Fetch data from a remote API using curl

```sql
select 'card' as component;
select value->>'name' as title, value->>'email' as description
from json_each(sqlpage.exec('curl', 'https://jsonplaceholder.typicode.com/users'));
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
  - When `sqlpage.exec(...)` is a standalone selected column, the command runs once per returned row. Use `SET command_result = sqlpage.exec(...)` first if the command should run only once for the page.
