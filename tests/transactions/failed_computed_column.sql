create temporary table query_event_rollback(value integer);
begin;
insert into query_event_rollback values (1);
select sqlpage.fetch(url) as value from (select 'invalid URL' as url) source_rows;
