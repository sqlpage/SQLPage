drop table if exists files_to_read;
create table files_to_read(filepath varchar(100));
insert into files_to_read(filepath) values ('tests/it_works.txt');
select 'It works !' as expected,
    sqlpage.read_file_as_text(filepath) as actual
from files_to_read;
