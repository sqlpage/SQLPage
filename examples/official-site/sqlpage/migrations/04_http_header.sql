-- Insert the http_header component into the component table
INSERT INTO component (name, description, icon)
VALUES (
        'http_header',
        'An advanced component to set arbitrary HTTP headers: can be used to set a custom caching policy to your pages, or implement custom redirections, for example.
        If you are a beginner, you probably don''t need this component.

        When used, this component has to be the first component in the page, because once the page is sent to the browser, it is too late to change the headers.

        HTTP headers are additional pieces of information sent with responses to web requests that provide instructions
        or metadata about the data being sent — for example,
        setting cache control directives to control caching behavior
        or specifying the content type of a response.
        
        Any valid HTTP header name can be used as a top-level parameter for this component.
        The examples shown here are just that, examples; and you can create any custom header
        if needed simply by declaring it.
        
        If your header''s name contains a dash or any other special character,
        you will have to use your database''s quoting mechanism to declare it.
        In standard SQL, you can use double quotes to quote identifiers (like "X-My-Header"),
        in Microsoft SQL Server, you can use square brackets (like [X-My-Header]).
        ',
        'world-www'
    );
-- Insert the parameters for the http_header component into the parameter table
INSERT INTO parameter (
        component,
        name,
        description,
        type,
        top_level,
        optional
    )
VALUES (
        'http_header',
        'Cache-Control',
        'Directives for how long the page should be cached by the browser. Set this to max-age=N to keep the page in cache for N seconds.',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'http_header',
        'Content-Disposition',
        'Provides instructions on how the response content should be displayed or handled by the client, such as inline or as an attachment.',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'http_header',
        'Location',
        'Specifies the URL to redirect the client to, usually used in 3xx redirection responses.',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'http_header',
        'Set-Cookie',
        'Sets a cookie in the client browser, used for session management and storing user-related information.',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'http_header',
        'Access-Control-Allow-Origin',
        'Specifies which origins are allowed to access the resource in a cross-origin request, used for implementing Cross-Origin Resource Sharing (CORS).',
        'TEXT',
        TRUE,
        TRUE
    );

-- Insert an example usage of the http_header component into the example table
INSERT INTO example (component, description, properties)
VALUES (
        'http_header',
        'Set cache control directives for caching behavior. In this example, the response can be cached by the browser
        and served from the cache for up to 600 seconds (10 minutes) after it is first requested.
        During that time, even if the cached response becomes stale (outdated), the browser can still use it (stale-while-revalidate)
        for up to 3600 seconds (1 hour) while it retrieves a fresh copy from the server in the background.
        If there is an error while trying to retrieve a fresh copy from the server,
        the browser can continue to serve the stale response for up to 86400 seconds (24 hours) (stale-if-error) instead of showing an error page.
        This caching behavior helps improve the performance and responsiveness of the website by reducing the number of requests made to the server
        and allowing the browser to serve content from its cache when appropriate.',
        JSON(
            '[{
                    "component": "http_header",
                    "Cache-Control": "public, max-age=600, stale-while-revalidate=3600, stale-if-error=86400"
        }]'
        )
    ),
    (
        'http_header',
        'Redirect the user to another page. In this example, the user is redirected to a file named another-page.sql at the root of the website. The current page will not be displayed at all.
        This is useful in particular for content creation pages that contain only INSERT statements, because you can redirect the user to the page that lists the content after it has been created.',
        JSON(
            '[{
                    "component": "http_header",
                    "Location": "/another-page.sql"
            }]'
        )
    ),
    (
        'http_header',
        'Set a custom non-standard header for the response. In this example, the response will include a custom header named X-My-Header with the value "my value".',
        JSON(
            '[{
                    "component": "http_header",
                    "X-My-Header": "my value"
            }]'
        )
    );