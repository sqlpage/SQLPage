select 'shell' as component, 'Customer tracker' as title, '' as footer,
    '/brand-demo/style.css' as css, '/brand-demo/symbol.svg' as favicon, '/brand-demo/symbol.svg' as image;
select 'form' as component, 'Add a customer' as title,
    'save_customer.sql' as action, 'Save customer' as validate;
select 'name' as name, 'Customer name' as label,
    true as required, 80 as maxlength;
select 'plan' as name, 'Plan' as label,
    'Pro' as value, true as required, 40 as maxlength;
select 'seats' as name, 'Seats' as label,
    'number' as type, 5 as value, 1 as min, 100 as max;
