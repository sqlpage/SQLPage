-- this test is skipped on MSSQL because its json_object function has a different syntax
select 'columns' as component;
select
	JSON_OBJECT('description', 'It works !') as item,
	JSON_OBJECT('description', 'It works !') as item
;