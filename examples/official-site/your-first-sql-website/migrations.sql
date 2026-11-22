select 'http_header' as component,
	'public, max-age=300, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control",
	'<https://sql-page.com/your-first-sql-website/migrations>; rel="canonical"' as "Link";

select 'dynamic' as component, sqlpage.run_sql('sqlpage/docs-shell.sql', json_object(
    'title', 'Database migrations | SQLPage',
    'description', 'Manage your database schema with SQLPage using migrations.',
    'section', 'Guides', 'current', 'Database migrations'
)) as properties;

-- Article by Matthew Larkin
select 'text' as component,
    sqlpage.read_file_as_text('your-first-sql-website/migrations.md') as contents_md,
    true as article;
