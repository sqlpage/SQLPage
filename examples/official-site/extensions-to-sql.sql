select 'http_header' as component,
	'public, max-age=300, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control";

select 'dynamic' as component, sqlpage.run_sql('sqlpage/docs-shell.sql', json_object(
    'title', 'SQL and variables | SQLPage',
    'section', 'Guides', 'current', 'SQL and variables'
)) as properties;

select 'title' as component, 'SQL and variables' as contents;

-- Article by Matthew Larkin
select 'text' as component,
    sqlpage.read_file_as_text('extensions-to-sql.md') as contents_md,
    true as article;

select 'docs-links' as component, 'Related reference' as title, 'reference' as id;
select 'Built-in functions' as title, 'function' as icon, '/functions.sql' as link,
    'Find signatures and examples for the sqlpage.* functions you can call in a query.' as description;
select 'Form parameters' as title, 'forms' as icon, '/component.sql?component=form' as link,
    'Set up the fields that supply POST parameters to your page.' as description;
