select 'http_header' as component,
    'public, max-age=600, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control",
    '<https://sql-page.com/>; rel="canonical"' as "Link";

-- A full document template uses an empty shell rather than the default site chrome.
select 'shell-empty' as component;
select 'landing-page' as component,
    sqlpage.read_file_as_text('landing-demos/table.sql') as demo_source;
