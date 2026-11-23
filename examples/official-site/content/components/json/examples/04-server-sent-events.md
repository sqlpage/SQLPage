## Access query results in real-time with server-sent events: `'sse' as type`

Using server-sent events, you can stream large query results to the client in real-time,
row by row.

This allows building sophisticated dynamic web applications that will start processing and displaying
the first rows of data in the browser while the database server is still processing the end of the query.

### SQL

```sql
select 'json' AS component, 'sse' AS type;
select * from users;
```

### JavaScript

```javascript
const eventSource = new EventSource("users.sql");
eventSource.onmessage = function (event) {
    const user = JSON.parse(event.data);
    console.log(user.username);
}
eventSource.onerror = () => eventSource.close(); // do not reconnect after reading all the data
```
