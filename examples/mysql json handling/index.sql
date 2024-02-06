select 'form' as component, 'Create a new Group' as title, 'Create' as validate;
select 'Name' as name;

insert into groups(name) select :Name where :Name is not null;

select 'list' as component, 'Groups' as title, 'No group yet' as empty_title;
select name as title from groups;

select 'form' as component, 'Add a user' as title, 'Add' as validate;
select 'UserName' as name, 'Name' as label;
select 
    'Memberships[]'  as name,
    'Group memberships' as label,
    'select' as type,
    TRUE     as multiple,
    'press ctrl to select multiple values' as description,
    json_arrayagg(json_object("label", name, "value", id)) as options
from groups;

insert into users(name) select :UserName where :UserName is not null;
insert into group_members(group_id, user_id)
select group_name, last_insert_id()
from json_table(:Memberships, '$[*]' columns (
    group_name int path '$'
)) as json_elems
where :Memberships is not null;

select 'list' as component, 'Users' as title, 'No user yet' as empty_title;
select
    users.name as title,
    group_concat(groups.name) as description
from users
left join group_members on users.id = group_members.user_id
left join groups on groups.id = group_members.group_id
group by users.id, users.name;