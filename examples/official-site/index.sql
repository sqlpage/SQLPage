select 'http_header' as component,
    'public, max-age=600, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control",
    '<https://sql-page.com/>; rel="canonical"' as "Link";

select 'shell-home' as component;
