insert or replace into blog_posts (id, title, content) 
select $id, :title, :content
where $id is not null and :title is not null and :content is not null
returning 'redirect' as component, 'post?id=' || $id as link;

select 'shell' as component,
    'Edit blog post' as title,
    '/rich_text_editor.js' as javascript_module;


select 'form' as component, 'Update' as validate;

with post as (
    select title, content
    from blog_posts
    where id = $id
),
fields as (
    select json_object(
        'name', 'title', 
        'label', 'Blog post title',
        'value', title
    ) as props
    from post
    union all
    select json_object(
        'name', 'content', 
        'type', 'textarea', 
        'label', 'Your blog post here', 
        'value', content, 
        'required', true
    )
    from post
)
select 'dynamic' as component, json_group_array(props) as properties from fields;