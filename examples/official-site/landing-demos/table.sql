select 'text' as component, 'Customers' as title;
select 'table' as component, 'landing-customers' as id,
    true as sort, true as search, 'status' as markdown;
select name, plan, status, seats, lower(status) as _sqlpage_css_class
from customers
order by id;
