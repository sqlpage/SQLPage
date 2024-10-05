-- this test is skipped on MSSQL and postgres because their json_object function has a different syntax
select 'columns' as component;
select
	JSON_OBJECT('description', 'It works !') as item,
	JSON_OBJECT('description', 'It works !') as item
;