SELECT 'list' AS component,
    'Navigation' AS title;

SELECT
    column1 AS title, column2 AS link, column3 AS description_md
FROM (VALUES
    ('Link to arbitrary path', '/api/does/not/actually/exist', 'Covered by `api/404.sql`'),
    ('Link to arbitrary file', '/api/nothing.png', 'Covered by `api/404.sql`'),
    ('Link to non-existing .sql file', '/api/inexistent.sql', 'Covered by `api/404.sql`'),
    ('Link to 404 handler', '/api/404.sql', 'Actually `api/404.sql`'),
    ('Link to API landing page', '/api', 'Covered by `api/index.sql`'),
    ('Link to arbitrary broken path', '/backend/does/not/actually/exist', 'Not covered by anything, will yield a 404 error')
);

SELECT 'text' AS component, 
    '
# Overview

This demo shows how a `404.sql` file can serve as a fallback error handler. Whenever a `404 Not
Found` error would be emitted, instead a dedicated `404.sql` is called (if it exists) to serve the
request. This is usefull in two scenarios:

1. Providing custom 404 error pages.
2. To provide content under dynamic paths.

The former use-case is primarily of cosmetic nature, it allows for more informative, customized
failure modes, enabling better UX. The latter use-case opens the door especially for REST API
design, where dynamic paths are often used to convey arguments, i.e. `/api/resource/5` where `5` is
the id of a resource.


# Fallback Handler Selection

When a normal request to either a `.sql` or a static file fails with `404`, the `404` error is
intercepted. The reuquest path''s target directory is scanned for a `404.sql`. If it exists, it is
called. Otherwise, the parent directory is scanned for `404.sql`, which will be called if it exists.
This search traverses up until it reaches the `web_root`. If even the webroot does not contain a
`404.sql`, then the original `404` error is served as response to the HTTP client.

The fallback handler is not recursive; i.e. if anything causes another `404` during the call to a
`404.sql`, then the request fails (emitting a `404` response to the HTTP client).
     ' AS contents_md;
