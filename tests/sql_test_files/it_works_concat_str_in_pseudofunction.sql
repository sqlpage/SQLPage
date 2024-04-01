select 'text' as component,
    'With "||": ' ||
    CASE sqlpage.url_encode('/' || $x)
        WHEN '%2F1' THEN 'It works !'
        ELSE 'Error: "/1" should be urlencoded to "%2F1"'
    END
    || ' | With CONCAT: ' ||
    CASE sqlpage.url_encode(CONCAT('/', $x)) -- $x is set to '1' in the test
        WHEN '%2F1' THEN 'With CONCAT: It works !'
        ELSE 'Error: "/1" should be urlencoded to "%2F1"'
    END
    || ' | With a null value: ' ||
    CASE sqlpage.url_encode(CONCAT('/', $thisisnull)) IS NULL
        WHEN true THEN 'With a null value: It works !'
        ELSE 'Error: a null value concatenated with "/" should be null, and urlencoded to NULL'
    END
    AS contents;