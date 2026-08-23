-- https://github.com/sqlpage/SQLPage/issues/1154
drop table if exists variable_limit_offset_t;
create table variable_limit_offset_t(id int primary key, v varchar(10));
insert into variable_limit_offset_t values (1,'a'),(2,'It works !'),(3,'c');
SET lim = 1; SET off = 1;
select 'It works !' as expected, v as actual from variable_limit_offset_t order by id limit $lim offset $off;
