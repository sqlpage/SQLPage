Create a cookie named `username` with the value `John Doe`...

```sql
SELECT 'cookie' as component,
        'username' as name,
        'John Doe' as value
        FALSE AS secure; -- You can remove this if the site is served over HTTPS.
```

and then display the value of the cookie using the [`sqlpage.cookie`](functions.sql?function=cookie) function:

```sql
SELECT 'text' as component,
         'Your name is ' || COALESCE(sqlpage.cookie('username'), 'not known to us');
```
