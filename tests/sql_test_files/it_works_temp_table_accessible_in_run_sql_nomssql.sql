-- Doesnt work on mssql because it does not support "create temporary table"
create temporary table temp_t(x text);
insert into temp_t(x) values ('It works !');
select 'dynamic' as component, sqlpage.run_sql('tests/sql_test_files/select_temp_t.sql') AS properties;