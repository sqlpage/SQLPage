set two = 2;
select 'text' as component,
    CASE
        WHEN $two = '2' -- All variables are strings
        THEN
            'It works !'
        ELSE
            'error: expected "2", got: ' || COALESCE($two, 'null')
    END as contents;
