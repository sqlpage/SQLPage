-- Documentation for the RSS component
INSERT INTO component (name, description, icon, introduced_in_version) VALUES (
    'html',
    'Include raw HTML in the output. For advanced users only. Use this component to create complex layouts or to include external content.
    Be very careful when using this component with user-generated content, as it can lead to security vulnerabilities.
    Use this component only if you are familiar with the security implications of including raw HTML, and understand the risks of cross-site scripting (XSS) attacks.',
    'html',
    '0.25.0'
);

INSERT INTO parameter (component,name,description,type,top_level,optional) VALUES (
    'html',
    'html',
    'Raw HTML content to include in the page. This will not be sanitized or escaped. If you include content from an untrusted source, your page will be vulnerable to cross-site scripting attacks.',
    'TEXT',
    TRUE,
    TRUE
),(
    'html',
    'html',
    'Raw HTML content to include in the page. This will not be sanitized or escaped. If you include content from an untrusted source, your page will be vulnerable to cross-site scripting attacks.',
    'TEXT',
    FALSE,
    TRUE
);

-- Insert example(s) for the component
INSERT INTO example (component, description, properties) VALUES (
    'html',
    'Include a simple HTML snippet. In this example, the HTML code is hardcoded in the SQL query, so it is safe. You should never include data that may be manipulated by a user in the HTML content.
    ',
    JSON('[{
        "component": "html",
        "html": "<h1 class=\"text-blue\">This text is safe because it is <mark>hardcoded</mark>!</h1>"
    }]')
),
(
    'html',
    'Include multiple html snippets as row-level parameters. Again, be careful what you include in the HTML content. If the data comes from a user, it can be manipulated to include malicious code.',
    JSON('[{"component":"html"},
    {"html":"<h1>First snippet</h1>"},
    {"html":"<h2>Second snippet</h2>"},
    {"html":"<h3>Third snippet</h3>"}]')
)
;