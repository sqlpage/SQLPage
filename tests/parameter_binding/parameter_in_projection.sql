DROP TABLE IF EXISTS sqlpage_param_projection;
CREATE TEMPORARY TABLE sqlpage_param_projection(username VARCHAR(100) NOT NULL);

INSERT INTO sqlpage_param_projection (username)
SELECT $x
WHERE $x IS NOT NULL;

select 'text' as component, max(username) as contents from sqlpage_param_projection;
