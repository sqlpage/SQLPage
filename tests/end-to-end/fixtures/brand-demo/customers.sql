select 'text' as component,
    'Customers' as title;
select 'table' as component,
    true as search, true as sort;
select name, plan, status, seats
from brand_demo_customers
order by id;
