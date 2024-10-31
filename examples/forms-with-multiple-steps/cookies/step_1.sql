select
    'form' as component,
    'step_2.sql' as action;

select
    'name' as name,
    true as required,
    sqlpage.cookie ('name') as value;