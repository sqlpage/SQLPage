-- syntax is valid in SQLite, PostgreSQL and SQLServer
-- no cast as varchar in MySQL
select 'text' as component, CAST('It works !' as varchar(100)) as contents;
