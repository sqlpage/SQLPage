select 'text' as component, 
    CASE sqlpage.variables()
        WHEN '{"x":"1"}' THEN 'It works !'
        ELSE 'It failed ! Expected {"x":"1"}, got ' || sqlpage.variables()
    END
    AS contents;