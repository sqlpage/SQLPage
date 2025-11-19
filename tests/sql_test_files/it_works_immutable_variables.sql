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
    WHEN json_extract(sqlpage.variables('get'), '$.x') = '1' THEN 'It works !'
    ELSE 'FAIL: variables(''get'') should return only URL parameters'
END AS contents;

SELECT CASE
    WHEN json_extract(sqlpage.variables('set'), '$.x') = 'set_value' AND
         json_extract(sqlpage.variables('set'), '$.set_only') = 'only_in_set' THEN 'It works !'
    ELSE 'FAIL: variables(''set'') should return only SET variables'
END AS contents;

SELECT CASE
    WHEN json_extract(sqlpage.variables(), '$.x') = 'set_value' AND
         json_extract(sqlpage.variables(), '$.set_only') = 'only_in_set' THEN 'It works !'
    ELSE 'FAIL: variables() should merge all with SET taking precedence'
END AS contents;

