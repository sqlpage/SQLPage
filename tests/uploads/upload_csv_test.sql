drop table if exists sqlpage_people_test_table;
create table sqlpage_people_test_table(name text, age text);
copy sqlpage_people_test_table(name, age) from 'people_file' with (format csv, header true);
select 'text' as component,
    name || ' is ' || age || ' years old. ' as contents
from sqlpage_people_test_table;