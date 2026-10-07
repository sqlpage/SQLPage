SELECT 'form' AS component, concat($instance, '-form') AS id, concat($instance, ' form') AS title, TRUE AS auto_submit;
SELECT concat($instance, '_value') AS name, concat($instance, ' value') AS label;
SELECT concat($instance, '_select') AS name, concat($instance, ' select') AS label,
    'select' AS type, TRUE AS searchable, '[{"label":"One","value":"one"}]' AS options;

SELECT 'table' AS component, concat($instance, '-table') AS id, TRUE AS search, TRUE AS sort;
SELECT 'Beta' AS name UNION ALL SELECT 'Alpha' AS name;

SELECT 'chart' AS component, concat($instance, '-chart') AS id, 'column' AS type;
SELECT 'A' AS x, 2 AS y;

SELECT 'facet' AS component, TRUE AS compact, concat($instance, ' choices') AS dropdown_title;
SELECT 'One' AS title, '#one' AS link;

SELECT 'modal' AS component, concat($instance, '-modal') AS id, concat($instance, ' dialog') AS title;
SELECT 'Fragment dialog contents' AS contents;

SELECT 'toast' AS component, concat($instance, '-toast') AS id, concat($instance, ' notification') AS title, 0 AS duration;
