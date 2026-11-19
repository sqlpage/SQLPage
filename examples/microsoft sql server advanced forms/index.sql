select 'form' as component, 'Create a new Group' as title, 'Create' as validate;
select 'Name' as name, 255 as maxlength;
select 'GroupDescription' as name, 'Description (optional)' as label, 80 as maxlength;

set GroupDescription = nullif(:GroupDescription, N'');
insert into groups(name, description)
select :Name, :GroupDescription
where :Name is not null;

select 'list' as component, 'Groups' as title, 'No group yet' as empty_title;
select name as title, description from groups;

select 'form' as component, 'Add a user' as title, 'Add' as validate;
select 'UserName' as name, 'Name' as label;
select 
    'Memberships[]'  as name,
    'Group memberships' as label,
    'select' as type,
    1     as multiple,
    'press ctrl to select multiple values' as description,
    (
        SELECT name as label, id as value
        FROM groups
        FOR JSON PATH -- this builds a JSON array of objects
    ) as options;

exec dbo.onboard_user :UserName, :Memberships;

select 'list' as component, 'Users' as title, 'No user yet' as empty_title;
select
    users.name as title,
    string_agg(cast(groups.name as nvarchar(max)), ', ')
        within group (order by groups.id) as description
from users
left join group_members on users.id = group_members.user_id
left join groups on groups.id = group_members.group_id
group by users.id, users.name;
