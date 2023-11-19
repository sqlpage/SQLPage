drop table if exists my_tmp_store;
create table my_tmp_store(x varchar(100));

insert into my_tmp_store(x) values ('It works !');

select 'card' as component;
select x as description from my_tmp_store;