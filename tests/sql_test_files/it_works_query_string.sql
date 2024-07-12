SELECT
    'text' AS component,
    CASE sqlpage.link('', sqlpage.variables('get'))
        WHEN '?x=1' THEN
            'It works !'
        ELSE
            'Expected "?x=1"'
    END AS contents
;