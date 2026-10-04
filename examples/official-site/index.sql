select 'http_header' as component,
    'public, max-age=600, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control",
    '<https://sql-page.com/>; rel="canonical"' as "Link",
    -- The scene uses pinned Three.js ES modules; keep this exception on the home page.
    'script-src ''self'' https://cdn.jsdelivr.net' as "Content-Security-Policy";

select 'shell-home' as component;
