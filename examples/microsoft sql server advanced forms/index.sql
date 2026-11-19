select 'form' as component, 'Create a new Group' as title, 'Create' as validate;
select 'Name' as name, 255 as maxlength;
select 'GroupDescription' as name, 'Description (optional)' as label, 80 as maxlength;

set GroupDescription = nullif(:GroupDescription, N'');
insert into dbo.groups (name, description)
output
    'alert' as component,
    'success' as color,
    'Group created' as title,
    inserted.name as description,
    concat('#group-', inserted.id) as link,
    'View group' as link_text
select :Name, :GroupDescription
where :Name is not null;

select 'list' as component, 'Groups' as title, 'No group yet' as empty_title;
select concat('group-', id) as id, name as title, description
from dbo.groups
order by id;

select 'form' as component, 'Add a user' as title, 'Add' as validate;
select 'UserName' as name, 'Name' as label;
select
    'Memberships[]' as name,
    'Group memberships' as label,
    'select' as type,
    1 as multiple,
    (
        select name as label, id as value
        from dbo.groups
        order by id
        for json path
    ) as options;

if :UserName is not null
begin
    set nocount on;
    set xact_abort on;

    begin transaction;

    insert into dbo.users (name) values (:UserName);
    declare @user_id int = scope_identity();

    insert into dbo.group_members (group_id, user_id)
    select group_id, @user_id
    from openjson(:Memberships) with (group_id int '$');

    commit transaction;
end;

select 'list' as component, 'Users' as title, 'No user yet' as empty_title;
select
    users.name as title,
    memberships.description,
    iif(memberships.description is null, 'No groups assigned', null) as description_md
from dbo.users
cross apply (
    select string_agg(cast(groups.name as nvarchar(max)), N', ')
        within group (order by groups.id) as description
    from dbo.group_members
    join dbo.groups on groups.id = group_members.group_id
    where group_members.user_id = users.id
) as memberships
order by users.id;
