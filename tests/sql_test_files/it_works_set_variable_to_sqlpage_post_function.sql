set my_var = sqlpage.url_encode(UPPER('a'));
select 'text' as component, 
    CASE $my_var
        WHEN 'A' THEN 'It works !'
        ELSE 'It failed !'
    END
    AS contents;