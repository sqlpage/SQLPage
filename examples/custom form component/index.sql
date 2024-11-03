-- include the common menu
select 'dynamic' as component, sqlpage.run_sql('shell.sql') as properties;

-- Call our custom component from ./sqlpage/templates/dual-list.handlebars
select
    'dual-list' as component,
    'form_action.sql' as action;

-- This SQL query returns the list of users, with a boolean indicating if they are in the group
select
    id,
    name as label,
    group_members.group_id is not null as selected
from
    users
    left join group_members on users.id = group_members.user_id
    and group_members.group_id = 1;