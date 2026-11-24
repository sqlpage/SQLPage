## Send query results as a single JSON array: `'array' as type`

The default `array` mode sends the query results as a single JSON array.

If a query returns an error, the array will contain an object with an `error` property.

If multiple queries are executed, all query results will be concatenated into a single array
of heterogeneous objects.

### SQL

```sql
select 'json' AS component;
select * from users;
```

### Result

```json
[
    {"username":"James","userid":1},
    {"username":"John","userid":2}
]
```

Clients can also receive JSON or JSON Lines automatically by requesting the same SQL file with an HTTP `Accept` header such as `application/json` or `application/x-ndjson` when the component is omitted, for example:

```
curl -H "Accept: application/json" http://localhost:8080/users.sql
```
