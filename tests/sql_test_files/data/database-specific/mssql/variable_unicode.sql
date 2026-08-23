-- MSSQL nvarchar with non-ASCII must not be mangled by CAST to VARCHAR
drop table if exists variable_unicode_t;
create table variable_unicode_t(name nvarchar(100));
insert into variable_unicode_t values (N'日本語');
SET x = N'日本語';
select N'日本語' as expected, name as actual from variable_unicode_t where name = $x;
