select 'text' as component, 
    CASE sqlpage.protocol()
        WHEN 'http' THEN 'It works !'
        ELSE 'It failed ! Expected "http", got "' || sqlpage.protocol() || '"".'
    END
    AS contents;