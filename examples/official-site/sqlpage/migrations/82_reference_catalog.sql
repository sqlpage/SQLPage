-- Short catalog descriptions help readers choose a page before opening its full reference.
CREATE TABLE reference_category (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    sort_order INTEGER NOT NULL
);

INSERT INTO reference_category (id, title, sort_order) VALUES
    ('display-data', 'Display data', 1),
    ('forms-and-feedback', 'Forms and feedback', 2),
    ('navigation', 'Navigation', 3),
    ('page-layout', 'Page layout and content', 4),
    ('http-responses', 'HTTP responses', 5),
    ('server-logic', 'Server logic', 6),
    ('request-data', 'Request data', 1),
    ('links-and-routing', 'Links and routing', 2),
    ('files-and-uploads', 'Files and uploads', 3),
    ('identity-and-security', 'Identity and security', 4),
    ('external-services', 'External services', 5),
    ('server-and-runtime', 'Server and runtime', 6);

ALTER TABLE component ADD COLUMN catalog_category TEXT REFERENCES reference_category(id);
ALTER TABLE component ADD COLUMN catalog_summary TEXT;
ALTER TABLE sqlpage_functions ADD COLUMN catalog_category TEXT REFERENCES reference_category(id);
ALTER TABLE sqlpage_functions ADD COLUMN catalog_summary TEXT;
ALTER TABLE sqlpage_function_parameters ADD COLUMN optional BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE sqlpage_function_parameters SET optional = TRUE WHERE ("function", name) IN (
    ('exec', 'arguments...'),
    ('hmac', 'algorithm'),
    ('link', 'parameters'), ('link', 'fragment'),
    ('oidc_logout_url', 'redirect_uri'),
    ('persist_uploaded_file', 'destination_folder'), ('persist_uploaded_file', 'allowed_extensions'), ('persist_uploaded_file', 'mode'),
    ('persist_uploaded_files', 'destination_folder'), ('persist_uploaded_files', 'allowed_extensions'), ('persist_uploaded_files', 'mode'),
    ('run_sql', 'parameters'),
    ('set_variable', 'value'),
    ('variables', 'method')
);

WITH copy(name, category, summary) AS (VALUES
    ('big_number', 'display-data', 'Highlight a metric with a trend or progress indicator.'),
    ('card', 'display-data', 'Arrange records in cards with images, links, or embedded content.'),
    ('carousel', 'display-data', 'Show a slideshow of images with optional captions and controls.'),
    ('chart', 'display-data', 'Plot query results as line, area, bar, or pie charts.'),
    ('datagrid', 'display-data', 'Lay out labeled values in a compact grid.'),
    ('list', 'display-data', 'Present rows as a vertical list with optional links and icons.'),
    ('map', 'display-data', 'Plot locations and geographic shapes on an interactive map.'),
    ('table', 'display-data', 'Turn query results into a sortable, searchable table.'),
    ('timeline', 'display-data', 'Arrange events along a vertical timeline.'),
    ('tracking', 'display-data', 'Show a sequence of activity or status changes as colored segments.'),

    ('alert', 'forms-and-feedback', 'Draw attention to a message with a color and an icon.'),
    ('button', 'forms-and-feedback', 'Follow a link, download a file, or submit a form.'),
    ('empty_state', 'forms-and-feedback', 'Explain an empty view and offer the reader a next step.'),
    ('form', 'forms-and-feedback', 'Collect input with text fields, file uploads, and other controls.'),
    ('login', 'forms-and-feedback', 'Display a sign-in form for a username and password.'),
    ('modal', 'forms-and-feedback', 'Open a dialog over the page for details, forms, or media.'),
    ('toast', 'forms-and-feedback', 'Show a brief notification above the page.'),

    ('breadcrumb', 'navigation', 'Link the current page to its parent pages.'),
    ('facet', 'navigation', 'Let readers choose filters for a dataset.'),
    ('pagination', 'navigation', 'Link to other pages of a query result.'),
    ('status_bar', 'navigation', 'Show status messages and links in a horizontal bar.'),
    ('steps', 'navigation', 'Mark progress through the stages of a task.'),
    ('tab', 'navigation', 'Switch between related pages using labeled tabs.'),

    ('code', 'page-layout', 'Display source code with syntax highlighting.'),
    ('columns', 'page-layout', 'Compare plans, features, or options in side-by-side cards.'),
    ('divider', 'page-layout', 'Separate sections with a horizontal rule.'),
    ('foldable', 'page-layout', 'Let readers expand individual sections to see their contents.'),
    ('hero', 'page-layout', 'Introduce a page with a headline, media, and a call to action.'),
    ('html', 'page-layout', 'Insert your own HTML markup into the page.'),
    ('shell', 'page-layout', 'Set the page layout, navigation, branding, and footer.'),
    ('text', 'page-layout', 'Write paragraphs using plain text or Markdown.'),
    ('title', 'page-layout', 'Add a heading and choose its level.'),

    ('cookie', 'http-responses', 'Set a cookie in the visitor''s browser.'),
    ('csv', 'http-responses', 'Export query rows as a downloadable CSV file.'),
    ('download', 'http-responses', 'Send a file directly to the visitor.'),
    ('http_header', 'http-responses', 'Set response headers for caching, content types, and more.'),
    ('json', 'http-responses', 'Return query results as JSON for an API.'),
    ('redirect', 'http-responses', 'Send the visitor to another URL.'),
    ('rss', 'http-responses', 'Publish an RSS feed from query results.'),
    ('status_code', 'http-responses', 'Set the HTTP status code returned by a page.'),

    ('authentication', 'server-logic', 'Restrict access by checking a password against a stored hash.'),
    ('debug', 'server-logic', 'Inspect the values passed to a component as JSON.'),
    ('dynamic', 'server-logic', 'Render SQLPage components described by JSON data.'),
    ('log', 'server-logic', 'Write a message to the SQLPage server log.')
)
UPDATE component SET (catalog_category, catalog_summary) = (copy.category, copy.summary)
FROM copy WHERE component.name = copy.name;

WITH copy(name, category, summary) AS (VALUES
    ('client_ip', 'request-data', 'Get the IP address of the client making the request.'),
    ('cookie', 'request-data', 'Read a named cookie from the incoming request.'),
    ('header', 'request-data', 'Read the value of one HTTP request header.'),
    ('headers', 'request-data', 'Read all HTTP request headers as a JSON object.'),
    ('path', 'request-data', 'Get the URL path of the current request.'),
    ('protocol', 'request-data', 'Identify whether the request used HTTP or HTTPS.'),
    ('request_body', 'request-data', 'Read the raw request body as text.'),
    ('request_body_base64', 'request-data', 'Read a binary request body as a Base64 string.'),
    ('request_method', 'request-data', 'Get the HTTP method, such as GET or POST.'),
    ('variables', 'request-data', 'Read request or page variables as a JSON object.'),

    ('link', 'links-and-routing', 'Build a page URL with optional query parameters and a fragment.'),
    ('regex_match', 'links-and-routing', 'Match text against a regular expression and extract capture groups.'),
    ('set_variable', 'links-and-routing', 'Build a link to this page with a query parameter changed or removed.'),
    ('url_encode', 'links-and-routing', 'Encode text for use in a URL.'),

    ('persist_uploaded_file', 'files-and-uploads', 'Save an uploaded file and return its path.'),
    ('persist_uploaded_files', 'files-and-uploads', 'Save multiple uploads from one field and return their file details.'),
    ('read_file_as_data_url', 'files-and-uploads', 'Read a file as a data URL for embedding in a page.'),
    ('read_file_as_text', 'files-and-uploads', 'Read the text of a UTF-8 file from the web root.'),
    ('uploaded_file_mime_type', 'files-and-uploads', 'Read the media type supplied by the browser for an uploaded file.'),
    ('uploaded_file_name', 'files-and-uploads', 'Read the original filename supplied by the browser.'),
    ('uploaded_file_path', 'files-and-uploads', 'Get the temporary path of an uploaded file.'),

    ('basic_auth_password', 'identity-and-security', 'Read the password sent with HTTP Basic authentication.'),
    ('basic_auth_username', 'identity-and-security', 'Read the username sent with HTTP Basic authentication.'),
    ('hash_password', 'identity-and-security', 'Hash a password with Argon2id for storage.'),
    ('hmac', 'identity-and-security', 'Sign data with a shared secret using HMAC.'),
    ('oidc_logout_url', 'identity-and-security', 'Build a sign-out URL for an OpenID Connect session.'),
    ('random_string', 'identity-and-security', 'Generate a cryptographically secure random string.'),
    ('user_info', 'identity-and-security', 'Read a named claim for the signed-in user.'),
    ('user_info_token', 'identity-and-security', 'Read all claims from the user''s OpenID Connect ID token.'),

    ('fetch', 'external-services', 'Send an HTTP request and read the response body.'),
    ('fetch_with_meta', 'external-services', 'Send an HTTP request and read its status, headers, and body.'),
    ('send_mail', 'external-services', 'Send email with optional attachments through an SMTP server.'),

    ('configuration_directory', 'server-and-runtime', 'Locate the directory for SQLPage configuration and templates.'),
    ('current_working_directory', 'server-and-runtime', 'Get the working directory of the SQLPage server process.'),
    ('environment_variable', 'server-and-runtime', 'Read a named environment variable from the server.'),
    ('exec', 'server-and-runtime', 'Run a program on the server and capture its output.'),
    ('run_sql', 'server-and-runtime', 'Run another SQLPage file and return its rows as JSON.'),
    ('version', 'server-and-runtime', 'Read the version of SQLPage serving the page.'),
    ('web_root', 'server-and-runtime', 'Locate the directory that holds the site''s public files.')
)
UPDATE sqlpage_functions SET (catalog_category, catalog_summary) = (copy.category, copy.summary)
FROM copy WHERE sqlpage_functions.name = copy.name;

CREATE VIEW sqlpage_functions_with_arguments AS
SELECT f.*, coalesce((
    SELECT group_concat(CASE WHEN optional THEN '[' || name || ']' ELSE name END, ', ' ORDER BY "index")
    FROM sqlpage_function_parameters WHERE "function" = f.name
), '') AS arguments
FROM sqlpage_functions AS f;

CREATE VIEW reference_catalog AS
WITH entries AS (
    SELECT 'component' AS kind, name, icon, catalog_category, catalog_summary, description,
        '/component.sql?component=' || name AS link,
        (SELECT count(*) FROM example WHERE example.component = component.name) AS example_count,
        NULL AS arguments
    FROM component
    UNION ALL
    SELECT 'function', name, icon, catalog_category, catalog_summary, description_md,
        '/functions.sql?function=' || name, NULL, arguments
    FROM sqlpage_functions_with_arguments
)
SELECT kind, name, icon, link, example_count, arguments,
    coalesce(category.id, 'other') AS category_id,
    coalesce(category.title, 'Other') AS category_title,
    coalesce(category.sort_order, 99) AS category_order,
    coalesce(catalog_summary, substr(trim(description, char(10) || ' #'), 1,
        instr(trim(description, char(10) || ' #') || char(10), char(10)) - 1)) AS summary
FROM entries
LEFT JOIN reference_category AS category ON category.id = entries.catalog_category;
