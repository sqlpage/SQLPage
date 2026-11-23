-- Preserve bookmarks to the database-filtered example projects.
select 'redirect' as component, sqlpage.link('/examples/',
    case when $db is not null then json_object('db', $db) end, 'projects') as link;
