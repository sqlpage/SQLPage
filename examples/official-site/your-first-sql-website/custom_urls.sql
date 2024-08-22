select 'http_header' as component,
    'public, max-age=300, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control";

select 'dynamic' as component, properties FROM example WHERE component = 'shell' LIMIT 1;

select 'hero' as component,
    'Custom URLs' as title,
    'SQLPage lets you customize responses to URLs that don''t match any file, using `404.sql`.' as description_md,
    'not_found.jpg' as image;

select 'text' as component, '
# Handling custom URLs

By default, SQLPage serves the file that matches the URL requested by the client.
If your users enter `https://example.com/about`, SQLPage will serve the file `about/index.sql` in your project.
If you create a file named `about.sql`, SQLPage will serve it when the user requests `https://example.com/about.sql`.

But what if you want to handle URLs that don''t match any file in your project ?
For example, what if you have a blog, and you want nice urls like `example.com/blog/my-trip-to-rome`,
but you don''t want to create a file for each blog post ?
By default, SQLPage would return a sad 404 error if you don''t have a file named `blog/my-trip-to-rome/index.sql`
in your project''s root directory.

But you can customize this behavior by creating a file named `404.sql` in your project.

## The 404.sql file

When SQLPage doesn''t find a file that matches the URL requested by the client, it will serve the file `404.sql` if it exists.

Since v0.28, when SQLPage receives a request for a URL like `https://example.com/a/b/c`, it will look for the file `a/b/c/index.sql` in your project,
and if it doesn''t find it, it will then search for, in order:
- `/a/b/404.sql`
- `/a/404.sql`
- `/404.sql`

## Basic routing example

So, you have a `blog_posts` table in your database, with columns `name`, and `content`.
You want to serve the content of the blog post with id `:id` when the user requests `example.com/blog/:id`.
You can do this by creating a `404.sql` file in the `blog` directory of your project:

```sql
-- blog/404.sql

-- Get the id from the URL
set name = substr(sqlpage.path(), 1+length(''/blog/''));

-- Get the blog post from the database
select ''text'' as component,
    content as contents_md
from blog_posts
where name = $name;
```

Now, when a user requests `example.com/blog/my-trip-to-rome`, SQLPage will serve the content of the blog post with name `my-trip-to-rome` from the `blog_posts` table.
' as contents_md;