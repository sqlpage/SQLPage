select 'shell-empty' as component,
    coalesce(sqlpage.request_body_base64(), 'NULL') as html; 