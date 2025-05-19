select 'shell' as component,
    title
from blog_posts
where id = $id;

select 'text' as component,
    true as article,
    content as contents_md
from blog_posts
where id = $id;

select 'list' as component;
select
    'Edit' as title,
    'pencil' as icon,
    'edit?id=' || $id as link;
