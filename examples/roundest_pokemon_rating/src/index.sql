select 'redirect' as component, '/populate.sql' as link
where not exists(select 1 from pokemon);

select 'pokemon' as component;

select dex_id as dexNumber, name 
from pokemon
order by random() limit 2;
