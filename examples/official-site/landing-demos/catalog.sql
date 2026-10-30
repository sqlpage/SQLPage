select 'landing-components' as component;
select name, icon, '/component.sql?component=' || sqlpage.url_encode(name) as link
from component
order by name;
