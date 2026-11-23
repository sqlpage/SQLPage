## Send a single JSON object: `'jsonlines' as type`

In `jsonlines` mode, each query result is rendered as a JSON object in a separate line,
without a top-level array.

If there is a single query result, the response will be a valid JSON object.
If there are multiple query results, you will need to parse each line of the response as a separate JSON object.

If a query returns an error, the response will be a JSON object with an `error` property.

### SQL

The following SQL creates an API endpoint that takes a `user_id` URL parameter
and returns a single JSON object containing the user's details, with one json object key per column in the `users` table.

```sql
select 'json' AS component, 'jsonlines' AS type;
select * from users where id = $user_id LIMIT 1;
```

> Note the `LIMIT 1` clause. The `jsonlines` type will send one JSON object per result row,
> separated only by a single newline character (\n).
> So if your query returns multiple rows, the result will not be a single valid JSON object,
> like most JSON parsers expect.

### Result

```json
{ "username":"James", "userid":1 }
```
