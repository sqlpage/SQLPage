select 'shell' as component,
    'Rich text editor' as title,
    '/rich_text_editor.js' as javascript_module;


select 'form' as component,
    'Create a new blog post' as title,
    'create_blog_post' as action,
    'Create' as validate;

select 'title' as name, 'Blog post title' as label, 'My new post' as value;
select 'content' as name, 'textarea' as type, 'Your blog post here' as label, 'Your blog post here' as value, true as required, $disabled is not null as disabled;

select 'list' as component,
    'Blog posts' as title;

select title, sqlpage.link('post', json_object('id', id)) as link
from blog_posts;
