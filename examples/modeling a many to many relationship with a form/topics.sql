SELECT * FROM sqlpage_shell LIMIT 1;

SELECT 'list' as component,
    'Topics' as title;
SELECT name as title,
    '/?topic=' || id as link,
    nb_posts || ' posts. '|| 
    COALESCE('Last post on *' || last_post || '*', '') as description_md,
    CASE
        WHEN last_post > date('now', '-2 days') THEN 'red'
        ELSE NULL
    END as color,
    icon,
    last_post > date('now', '-2 days') as active
FROM topic_with_stats;