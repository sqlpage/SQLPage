---
icon: variable
since: 0.11.0
category: server-and-runtime
summary: Read a named environment variable from the server.
parameters:
  - name: name
    type: TEXT
    required: true
    description: The name of the environment variable to read. Must be a literal string.
---

Returns the value of the given [environment variable](https://en.wikipedia.org/wiki/Environment_variable).

### Example

```sql
SELECT 'text' AS component;
SELECT 'The value of the HOME environment variable is ' AS contents;
SELECT sqlpage.environment_variable('HOME') as contents, true as code;
```
