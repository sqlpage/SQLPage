-- This updates our pokemon table with fresh data from pokeapi.co
insert into pokemon (dex_id, name)
select 
    cast(rtrim(substr(value->>'url', 
        length('https://pokeapi.co/api/v2/pokemon/') + 1), 
    '/') as integer) as dex_id,
    value->>'name' as name
from json_each(
    sqlpage.fetch('https://pokeapi.co/api/v2/pokemon?limit=100000&offset=0') -> 'results'
);

select 'redirect' as component, '/' as link;