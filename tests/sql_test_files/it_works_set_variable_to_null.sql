set i_am_null = NULL;
select 'text' as component,
    CASE
        WHEN $i_am_null IS NULL
        THEN
            'It works !'
        ELSE
            'error: expected null, got: ' || $i_am_null
    END as contents;
