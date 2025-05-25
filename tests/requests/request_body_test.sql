select 'shell-empty' as component,
    coalesce(sqlpage.request_body(), 'NULL') as html; 