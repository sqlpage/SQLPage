-- Adds a new entry in the clicks table
INSERT INTO clicks(click_time) VALUES (datetime('now'));

SELECT 'json' AS component,
        JSON_OBJECT(
            'total_clicks', (SELECT count(*) FROM clicks)
        ) AS contents;