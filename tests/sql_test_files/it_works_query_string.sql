SELECT
    'dynamic' AS component,
    CASE sqlpage.query_string()
        WHEN 'x=1' THEN
            '[{"component":"text"}, {"contents":"It works !"}]'
        ELSE
            '[{"component":"redirect", "link":"login.sql?' || sqlpage.query_string() || '"}]'
    END AS properties
;
