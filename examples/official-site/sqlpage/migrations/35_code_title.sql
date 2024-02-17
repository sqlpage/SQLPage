-- Documentation for the title component
INSERT INTO component (name, description, icon, introduced_in_version) VALUES (
    'title',
    'Defines HTML headings. The level 1 is used for the maximal size and the level 6 is used for the minimal size.',
    'letter-case-upper',
    '0.19.0'
);

INSERT INTO parameter (component,name,description,type,top_level,optional) VALUES (
    'title',
    'center',
    'Whether to center the title.',
    'BOOLEAN',
    TRUE,
    TRUE
),(
    'title',
    'contents',
    'A text to display.',
    'TEXT',
    TRUE,
    FALSE
),(
    'title',
    'level',
    'Set the heading level (default level is 1)',
    'NUMBER',
    TRUE,
    TRUE   
);

-- Insert example(s) for the component
INSERT INTO example(component, description, properties) VALUES (
    'title',
    'Displays several titles with different levels.',
    JSON(
        '[
            {"component":"title","contents":"Level 1"},
            {"component":"title","contents":"Level 2","level": 2},
            {"component":"title","contents":"Level 3","level": 3}
        ]'
    )
);

INSERT INTO example(component, description, properties) VALUES (
    'title',
    'Displays a centered title.',
    JSON(
        '[
            {"component":"title","contents":"Level 1","center": true}
        ]'
    )
);

-- Documentation for the code component
INSERT INTO component (name, description, icon, introduced_in_version) VALUES (
    'code',
    'Displays one or many blocks of code from a programming language or formated text as XML or JSON.',
    'code',
    '0.19.0'
);

INSERT INTO parameter (component,name,description,type,top_level,optional) VALUES (
    'code',
    'title',
    'Set the heading level (default level is 1)',
    'TEXT',
    FALSE,
    TRUE
),(
    'code',
    'contents',
    'A block of code.',
    'TEXT',
    FALSE,
    FALSE
),(
    'code',
    'description',
    'Description of the snipet of code.',
    'TEXT',
    FALSE,
    TRUE
),(
    'code',
    'description_md',
    'Rich text in the markdown format. Among others, this allows you to write bold text using **bold**, italics using *italics*, and links using [text](https://example.com).',
    'TEXT',
    FALSE,
    TRUE
),(
    'code',
    'language',
    'Set the programming language name.',
    'TEXT',
    FALSE,
    TRUE
);

-- Insert example(s) for the component
INSERT INTO example(component, description, properties) VALUES (
    'code',
    'Displays a block of HTML code.',
    JSON(
        '[
            {"component":"code"},
            {
                "title":"A HTML5 example",
                "language":"html",
                "description":"Here’s the very minimum that an HTML document should contain, assuming it has CSS and JavaScript linked to it.",
                "contents":"<!DOCTYPE html>\n<html lang=\"en\">\n\t<head>\n\t\t<meta charset=\"utf-8\">\n\t\t<title>title</title>\n\t\t<link rel=\"stylesheet\" href=\"style.css\">\n\t\t<script src=\"script.js\"></script>\n\t</head>\n\t<body>\n\t\t<!-- page content -->\n\t</body>\n</html>"
            }
        ]'
    )
);