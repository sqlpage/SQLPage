select 'shell' as component,
    title
from blog_posts
where id = $id;

select 'text' as component,
    true as article,
    content as contents_md
from blog_posts
where id = $id;
