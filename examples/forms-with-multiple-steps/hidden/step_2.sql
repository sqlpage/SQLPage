select
    'form' as component,
    'step_3.sql' as action;

select
    'email' as name,
    'email' as type,
    true as required,
    'you@example.com' as placeholder,
    'Hey ' || :name || '! what is your email?' as description;

with previous_answers(name, value) as (values ('name', :name))
select 'hidden' as type, name, value from previous_answers;
