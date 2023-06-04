INSERT INTO games(id)
VALUES(random())
RETURNING
    'http_header' as component,
    'game.sql?id='||id as "Location";

SELECT 'text' as component, 'redirecting to game...' as contents;