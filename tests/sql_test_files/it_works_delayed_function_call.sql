drop table if exists files_to_read;
create table files_to_read(filepath varchar(100));
insert into files_to_read(filepath) values ('tests/it_works.txt');
select 'text' as component, sqlpage.read_file_as_text(filepath) as contents from files_to_read;
