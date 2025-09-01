select 'http_header' as component,
	'public, max-age=300, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control";

select 'dynamic' as component, json_patch(json_extract(properties, '$[0]'), json_object(
    'title', 'SQLPage - Extensions to SQL'
)) as properties
FROM example WHERE component = 'shell' LIMIT 1;

-- Article by Matthew Larkin
select 'text' as component,
    sqlpage.read_file_as_text('extensions-to-sql.md') as contents_md,
    true as article;
