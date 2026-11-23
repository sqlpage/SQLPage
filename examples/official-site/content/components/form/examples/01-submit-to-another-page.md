---
rows:
  - component: form
    action: create_user.sql
  - name: username
---

The best way to manage forms in SQLPage is to create at least two separate files:

 - one that will contain the form itself, and will be loaded when the user visits the page,
 - one that will handle the form submission, and will redirect to whatever page you want to display after the form has been submitted.

For instance, if you were creating a form to manage a list of users, you could create:

 - a file named `users.sql` that would contain a list of users and a form to create a new user,
 - a file named `create_user.sql` that would insert the new user in the database, and then redirect to `users.sql`.

`create_user.sql` could contain the following sql statement to [safely](safety.sql) insert the new user in the database:

```sql
INSERT INTO users(name) VALUES(:username)
RETURNING 'redirect' AS component, 'users.sql' AS link
```

When loading the page, the value for `:username` will be `NULL` if no value has been submitted.
