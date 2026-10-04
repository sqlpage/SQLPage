select 'text' as component, 'Customers' as title;
select 'table' as component, 'landing-customers' as id,
    true as sort, true as search;
select 'Acme Corp' as name, 'Pro' as plan, 'Active' as status, 12 as seats
union all select 'Globex', 'Team', 'Active', 8
union all select 'Initech', 'Free', 'Trial', 3
union all select 'Umbrella', 'Pro', 'Late', 20;
