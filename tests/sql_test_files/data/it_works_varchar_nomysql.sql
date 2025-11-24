-- syntax is valid in SQLite, PostgreSQL and SQLServer
-- no cast as varchar in MySQL
select 'It works !' as expected,
    CAST('It works !' as varchar(100)) as actual;
