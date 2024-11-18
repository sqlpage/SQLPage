set my_var = sqlpage.url_encode(' ');
select 'text' as component, 
    CASE $my_var
        WHEN '%20' THEN 'It works !'
        ELSE 'It failed !'
    END
    AS contents;