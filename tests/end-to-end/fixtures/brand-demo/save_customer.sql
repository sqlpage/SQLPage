select 'status_code' as component, 400 as status
where sqlpage.request_method() <> 'POST'
   or length(trim(coalesce(:name, ''))) not between 1 and 80
   or length(trim(coalesce(:plan, ''))) not between 1 and 40
   or cast(coalesce(:seats, '0') as integer) not between 1 and 100;
INSERT INTO brand_demo_customers (name, plan, status, seats)
SELECT trim(:name), trim(:plan), 'Active', cast(:seats as integer)
WHERE sqlpage.request_method() = 'POST'
    AND length(trim(coalesce(:name, ''))) between 1 and 80
    AND length(trim(coalesce(:plan, ''))) between 1 and 40
    AND cast(coalesce(:seats, '0') as integer) between 1 and 100;
select 'redirect' as component, './' as link;
