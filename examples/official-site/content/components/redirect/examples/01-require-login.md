Redirect a user to the login page if they are not logged in:

```sql
SELECT 'redirect' AS component, 'login.sql' AS link
WHERE NOT EXISTS (SELECT 1 FROM login_session WHERE id = sqlpage.cookie('session_id'));
```
