select
    'form' as component,
    'finish.sql' as action;

select
    'age' as name,
    'number' as type,
    true as required,
    'How old are you, ' || :name || '?' as description;

with previous_answers(name, value) as (values ('name', :name), ('email', :email))
select 'hidden' as type, name, value from previous_answers;
