-- Variable compared to integer column must work without explicit cast
drop table if exists variable_integer_comparison_t;
create table variable_integer_comparison_t(id int primary key, name varchar(100));
insert into variable_integer_comparison_t values (1, 'It works !');
select 'It works !' as expected, name as actual from variable_integer_comparison_t where id = $x;
