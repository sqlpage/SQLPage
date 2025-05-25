set actual = sqlpage.link('', sqlpage.variables('get'));
set expected = '?x=1';
SELECT
    'text' AS component,
    CASE $actual
        WHEN $expected THEN
            'It works !'
        ELSE
            'Expected ' || COALESCE($expected, 'null') || ' but got ' || COALESCE($actual, 'null')
    END AS contents
;