insert into blog_posts (title, content)
values (:title, :content)
returning 
  'redirect' as component,
  'post?id=' || id as link;