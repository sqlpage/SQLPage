INSERT INTO games(id)
VALUES(random())
RETURNING
    'redirect' as component,
    CONCAT('game.sql?id=', id) as link;