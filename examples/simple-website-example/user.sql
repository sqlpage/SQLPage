SELECT 'text' as component,
    username as title,
    username || ' is an user on this site.

[Delete this user](delete.sql?id=' || id || ')

[Edit user](edit.sql?id=' || id || ')' as contents_md
FROM users
WHERE id = $id;