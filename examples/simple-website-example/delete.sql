DELETE FROM users
WHERE id = $id
RETURNING
   'text' AS component,
   '
The user ' || username || ' has been deleted.

[Back to the user list](index.sql)' as contents_md;