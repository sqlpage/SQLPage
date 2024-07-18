-- Documentation for the RSS component
INSERT INTO
    component (name, description, icon, introduced_in_version)
VALUES
    (
        'html',
        'Include raw HTML in the output. For advanced users only. Use this component to create complex layouts or to include external content.
    Be very careful when using this component with user-generated content, as it can lead to security vulnerabilities.
    Use this component only if you are familiar with the security implications of including raw HTML, and understand the risks of cross-site scripting (XSS) attacks.',
        'html',
        '0.25.0'
    );

INSERT INTO
    parameter (
        component,
        name,
        description,
        type,
        top_level,
        optional
    )
VALUES
    (
        'html',
        'html',
        'Raw HTML content to include in the page. This will not be sanitized or escaped. If you include content from an untrusted source, your page will be vulnerable to cross-site scripting attacks.',
        'TEXT',
        TRUE,
        TRUE
    ),
    (
        'html',
        'html',
        'Raw HTML content to include in the page. This will not be sanitized or escaped. If you include content from an untrusted source, your page will be vulnerable to cross-site scripting attacks.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'html',
        'text',
        'Text content to include in the page. This will be sanitized and escaped. Use this property to include user-generated content that should not contain HTML tags.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'html',
        'post_html',
        'Raw HTML content to include after the text content. This will not be sanitized or escaped. If you include content from an untrusted source, your page will be vulnerable to cross-site scripting attacks.',
        'TEXT',
        FALSE,
        TRUE
    );

-- Insert example(s) for the component
INSERT INTO
    example (component, description, properties)
VALUES
    (
        'html',
        'Include a simple HTML snippet. In this example, the HTML code is hardcoded in the SQL query, so it is safe. You should never include data that may be manipulated by a user in the HTML content.
    ',
        JSON (
            '[{
        "component": "html",
        "html": "<h1 class=\"text-blue\">This text is safe because it is <mark>hardcoded</mark>!</h1>"
    }]'
        )
    ),
    (
        'html',
        'Include multiple html snippets as row-level parameters. Again, be careful what you include in the HTML content. If the data comes from a user, it can be manipulated to include malicious code.',
        JSON (
            '[{"component":"html", "html":"<div class=\"d-flex gap-3 mb-4\" style=\"height: 40px\">"},
    {"html":"<div class=\"progress h-100\"><div class=\"progress-bar progress-bar-striped progress-bar-animated\" style=\"width: 10%\">10%</div></div>"},
    {"html":"<div class=\"progress h-100\"><div class=\"progress-bar progress-bar-striped progress-bar-animated  bg-danger\" style=\"width: 80%\">80%</div></div>"},
    {"html":"</div>"}
    ]'
        )
    ),
    (
        'html',
        'In order to include user-generated content that should be sanitized, use the `text` property instead of `html`. The `text` property will display the text as-is, without interpreting any HTML tags.',
        JSON (
            '
        [
            {"component": "html"},
            {
                "html": "<p>The following will be sanitized: <mark>",
                "text": "<script>alert(''Failed XSS attack!'')</script>",
                "post_html": "</mark>. Phew! That was close!</p>"
            }
        ]'
        )
    );