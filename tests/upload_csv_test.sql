create table people(name text, age text);
copy people(name, age) from 'people_file' with (format csv, header true);
select 'text' as component,
    name || ' is ' || age || ' years old. ' as contents
from people;