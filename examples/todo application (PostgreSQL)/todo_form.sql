
-- When the form is submitted, we insert the todo item into the database
-- or update it if it already exists
-- and redirect the user to the home page.
-- When the form is initially loaded, :todo is null,
-- nothing is inserted, and the 'redirect' component is not returned.
insert into todos(id, title)
select COALESCE($todo_id::int, nextval('todos_id_seq')), :todo -- $todo_id will be null if the page is accessed via the 'Add new todo' button (without a ?todo_id= parameter)
where :todo is not null -- only insert if the form was submitted
on conflict(id) do update set title = excluded.title
returning
    'redirect' as component,
    '/' as link;

-- The header needs to come before the form, but after the potential redirect
select 'dynamic' as component, sqlpage.run_sql('shell.sql') as properties;

-- The form needs to come AFTER the insert statement
-- because the insert statement will redirect the user to the home page if the form was submitted
select 
    'form'            as component,
    'Todo'            as title,
    (
        case when $todo_id is null then
            'Add new todo'
        else
            'Edit todo'
        end
    ) as validate;
select 
    'Todo item' as label,
    'todo' as name,
    'What do you have to do ?' as placeholder,
    (select title from todos where id = $todo_id::int) as value;