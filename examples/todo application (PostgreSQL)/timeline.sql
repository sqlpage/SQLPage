select
    'dynamic' as component,
    sqlpage.run_sql ('shell.sql') as properties;

select
    'timeline' as component;

SELECT
    title,
    'todo_form.sql?todo_id=' || id AS link,
    TO_CHAR (created_at, 'FMMonth DD, YYYY,  HH12:MI AM TZ') AS date,
    'calendar' AS icon,
    'green' AS color,
    CONCAT (
        EXTRACT(
            DAY
            FROM
                NOW () - created_at
        ),
        ' days ago'
    ) AS description
FROM
    todos
ORDER BY
    created_at DESC;