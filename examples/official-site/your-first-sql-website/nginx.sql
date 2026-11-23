select 'http_header' as component,
    'public, max-age=300, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control";

select 'dynamic' as component, sqlpage.run_sql('sqlpage/docs-shell.sql', json_object(
    'title', 'Reverse proxy | SQLPage',
    'section', 'Guides', 'current', 'Reverse proxy'
)) as properties;
select 'text' as component, true as article, sqlpage.read_file_as_text('your-first-sql-website/nginx.md') as contents_md;
