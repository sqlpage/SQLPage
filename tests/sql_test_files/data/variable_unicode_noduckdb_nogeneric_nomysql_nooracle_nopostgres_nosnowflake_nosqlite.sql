-- Comparing a variable to an nvarchar column must not mangle non-Latin
-- characters. SQLPage must not cast the variable to a narrow varchar type.
drop table if exists variable_unicode_t;
create table variable_unicode_t(name nvarchar(100));
insert into variable_unicode_t(name) values (N'日本語');

SET x = N'日本語';

select N'日本語' as expected, name as actual from variable_unicode_t where name = $x;
