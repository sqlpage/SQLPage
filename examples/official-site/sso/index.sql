select 'http_header' as component,
    'public, max-age=600, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control",
    '<https://sql-page.com/sso>; rel="canonical"' as "Link";

select 'dynamic' as component, properties FROM example WHERE component = 'shell' LIMIT 1;

select 'text' as component, sqlpage.read_file_as_text('sso/single_sign_on.md') as contents_md, true as article;