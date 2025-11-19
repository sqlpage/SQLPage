SET x = 'set_value';
SET set_only = 'only_in_set';

SELECT 'text' AS component;

SELECT CASE 
    WHEN $x = 'set_value' THEN 'It works !'
    WHEN $x = '1' THEN 'FAIL: SET variable should shadow URL param'
    ELSE 'FAIL: Unexpected value for $x: ' || COALESCE($x, 'NULL')
END AS contents;

SELECT CASE
    WHEN $set_only = 'only_in_set' THEN 'It works !'
    ELSE 'FAIL: SET-only variable not found'
END AS contents;

SELECT CASE
    WHEN sqlpage.variables('get') = '{"x":"1"}' THEN 'It works !'
    ELSE 'FAIL: variables(''get'') should return only URL parameters, got: ' || sqlpage.variables('get')
END AS contents;

SELECT CASE
    WHEN sqlpage.variables('set') = '{"x":"set_value","set_only":"only_in_set"}' THEN 'It works !'
    ELSE 'FAIL: variables(''set'') should return only SET variables, got: ' || sqlpage.variables('set')
END AS contents;

SELECT CASE
    WHEN sqlpage.variables() = '{"x":"set_value","set_only":"only_in_set"}' THEN 'It works !'
    ELSE 'FAIL: variables() should merge all with SET taking precedence, got: ' || sqlpage.variables()
END AS contents;

