select 'text' as component, 
    CASE sqlpage.url_encode('/')
        WHEN '%2F' THEN 'It works !'
        ELSE 'It failed !'
    END
    AS contents;