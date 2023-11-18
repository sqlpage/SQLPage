create table my_tmp_store(x text);

insert into my_tmp_store(x) values ('It works !');

select 'card' as component;
select x as description from my_tmp_store;
drop table my_tmp_store;