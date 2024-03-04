-- Documentation for the RSS component
INSERT INTO component (name, description, icon, introduced_in_version) VALUES (
    'rss',
    'Produces a data flow in the RSS format. To use this component, you must first returning an HTTP header with the "application/rss+xml" content type (see http_header component). Next, you must use the shell-empty component to avoid that SQLPage generates HTML code.',
    'rss',
    '0.20.0'
);

INSERT INTO parameter (component,name,description,type,top_level,optional) VALUES (
    'rss',
    'title',
    'Defines the title of the channel.',
    'TEXT',
    TRUE,
    FALSE
),(
    'rss',
    'link',
    'Defines the hyperlink to the channel.',
    'URL',
    TRUE,
    FALSE
),(
    'rss',
    'description',
    'Describes the channel.',
    'TEXT',
    TRUE,
    FALSE  
),(
    'rss',
    'title',
    'Defines the title of the item.',
    'TEXT',
    FALSE,
    FALSE  
),(
    'rss',
    'link',
    'Defines the hyperlink to the item',
    'URL',
    FALSE,
    FALSE 
),(
    'rss',
    'description',
    'Describes the item',
    'TEXT',
    FALSE,
    FALSE 
),(
    'rss',
    'pubdate',
    'Indicates when the item was published (RFC-822 date-time).',
    'TEXT',
    FALSE,
    TRUE 
);

-- Insert example(s) for the component
INSERT INTO example (component, description)
VALUES (
        'rss',
        '
### An RSS chanel about SQLPage latest news.

```sql
select ''http_header'' as component, ''application/rss+xml'' as content_type;
select ''shell-empty'' as component;
select
	''rss'' as component,
	''SQLPage blog'' as title,
	''https://sql.ophir.dev/blog.sql'' as link,
	''latest news about SQLpage'' as description;
select
	''Hello everyone !'' as title,
	''https://sql.ophir.dev/blog.sql?post=Come%20see%20me%20build%20twitter%20live%20on%20stage%20in%20Prague'' as link,
	''If some of you european SQLPagers are around Prague this december, I will be giving a talk about SQLPage at pgconf.eu on December 14th.'' as description;
select
	''3 solutions to the 3 layer problem'' as title,
	''https://sql.ophir.dev/blog.sql?post=3%20solutions%20to%20the%203%20layer%20problem'' as link,
	''Some interesting questions emerged from the article Repeating yourself.'' as description,
	''Mon, 04 Dec 2023 00:00:00 GMT'' as pubdate;
```
');