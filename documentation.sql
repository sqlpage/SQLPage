DROP TABLE IF EXISTS example;
DROP TABLE IF EXISTS parameter;
DROP TABLE IF EXISTS component;

CREATE TABLE component(
    name TEXT PRIMARY KEY,
    description TEXT NOT NULL,
    icon TEXT -- icon name from tabler icon
);

CREATE TABLE parameter(
    top_level BOOLEAN DEFAULT FALSE,
    name TEXT,
    component TEXT REFERENCES component(name) ON DELETE CASCADE,
    description TEXT NOT NULL,
    type TEXT,
    optional BOOLEAN DEFAULT FALSE,
    PRIMARY KEY (component, top_level, name)
);

CREATE TABLE example(
    component TEXT REFERENCES component(name) ON DELETE CASCADE,
    description TEXT,
    properties TEXT,
    FOREIGN KEY (component) REFERENCES component(name) ON DELETE CASCADE
);

INSERT INTO component(name, icon, description) VALUES
    ('list', 'list', 'A vertical list of items. Each item can be clickable and link to another page.');
INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'list', * FROM (VALUES
    -- top level
    ('title', 'Text header at the top of the list.', 'TEXT', TRUE, TRUE),
    -- item level
    ('title', 'Name of the list item, displayed prominently.', 'TEXT', FALSE, FALSE),
    ('description', 'A description of the list item, displayed as greyed-out text.', 'TEXT', FALSE, TRUE),
    ('link', 'An URL to which the user should be taken when they click on the list item.', 'URL', FALSE, TRUE),
    ('icon', 'An icon name (from tabler-icons.io) to display on the left side of the item.', 'TEXT', FALSE, TRUE),
    ('color', 'The name of a color, to be displayed as a dot near the list item contents.', 'TEXT', FALSE, TRUE),
    ('active', 'Whether this item in the list is considered "active". Active items are displayed more prominently.', 'BOOLEAN', FALSE, TRUE)
);

INSERT INTO component(name, icon, description) VALUES
    ('text', 'align-left', 'A paragraph of text. The entire component will render as a single paragraph, with each item being rendered as a span of text inside it, the styling of which can be customized using parameters.');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'text', * FROM (VALUES
    -- top level
    ('title', 'Text header before the paragraph.', 'TEXT', TRUE, TRUE),
    ('center', 'Whether to center the title.', 'BOOLEAN', TRUE, TRUE),
    ('width', 'How wide the paragraph should be, in characters.', 'INTEGER', TRUE, TRUE),
    -- item level
    ('contents', 'A span of text to display', 'TEXT', FALSE, FALSE),
    ('link', 'An URL to which the user should be taken when they click on this span of text.', 'URL', FALSE, TRUE),
    ('color', 'The name of a color for this span of text.', 'TEXT', FALSE, TRUE),
    ('underline', 'Whether the span of text should be underlined.', 'BOOLEAN', FALSE, TRUE),
    ('bold', 'Whether the span of text should be displayed as bold.', 'BOOLEAN', FALSE, TRUE),
    ('code', 'Use a monospace font. Useful to display the text as code.', 'BOOLEAN', FALSE, TRUE),
    ('italics', 'Whether the span of text should be displayed as italics.', 'BOOLEAN', FALSE, TRUE)
);

INSERT INTO example(component, description, properties) VALUES
    ('text', 'Rendering a simple text paragraph.', json('[{"component":"text"}, {"contents":"Blah", "bold":true}]'));

INSERT INTO component(name, icon, description) VALUES
    ('form', 'cursor-text', 'A series of input fields that can be filled in by the user. The form contents can be posted and handled by another SQLPage. The form contents are accessible from the target page as ($1->>''$.form.x'') for a form field named x.');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'form', * FROM (VALUES
    -- top level
    ('method', 'Set this to ''GET'' to pass the form contents directly as URL parameters, accessible from the target page as ($1->>''$.form.x'')', 'TEXT', TRUE, TRUE),
    ('action', 'An optional link to a target page that will handle the results of the form. By default the target page is the current page.', 'TEXT', TRUE, TRUE),
    ('title', 'A name to display at the top of the form.', 'TEXT', TRUE, TRUE),
    ('validate', 'The text to display in the button at the bottom of the form that submits the values.', 'TEXT', TRUE, TRUE),
    -- item level
    ('type', 'The type of input to use: text for a simple text field, number for field that accepts only numbers, checkbox or radio for a button that is part of a group specified in the ''name'' parameter.', 'TEXT', FALSE, FALSE),
    ('name', 'The name of the input field, that you can use in the target page to get the value the user entered for the field.', 'TEXT', FALSE, FALSE),
    ('label', 'A friendly name for the text field to show to the user.', 'TEXT', FALSE, TRUE),
    ('placeholder', 'A placeholder text that will be shown in the field when is is empty.', 'TEXT', FALSE, TRUE),
    ('value', 'A default value that will already be present in the field when the user loads the page.', 'TEXT', FALSE, TRUE),
    ('required', 'Set this to true to prevent the form contents from being sent if this field is left empty by the user.', 'BOOL', FALSE, TRUE),
    ('min', 'The minimum value to accept for an input of type number', 'NUMBER', FALSE, TRUE),
    ('max', 'The minimum value to accept for an input of type number', 'NUMBER', FALSE, TRUE),
    ('step', 'The increment of values in an input of type number. Set to 1 to allow only integers.', 'NUMBER', FALSE, TRUE),
    ('description', 'A helper text to display near the input field.', 'TEXT', FALSE, TRUE)
);

INSERT INTO component(name, icon, description) VALUES
    ('dynamic', 'repeat', 'A special component that can be used to render other components, the number and properties of which are not known in advance.');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'dynamic', * FROM (VALUES
    -- top level
    ('properties', 'A json object or array that contains the names and properties of other components', 'JSON', TRUE, TRUE)
);

INSERT INTO example(component, description, properties) VALUES
    ('dynamic', 'Rendering a text paragraph dynamically.', json('[{"component":"dynamic", "properties": "[{\"component\":\"text\"}, {\"contents\":\"Blah\", \"bold\":true}]"}]'));

select
    'SQLPage documentation' as title,
    '/' as link,
    'en' as lang,
    'SQLPage documentation' as description;


select 'text' as component, 'SQLPage documentation' as title;
select 'Building an application with SQLPage is quite simple.' ||
    'To create a new web page, just create a new SQL file. ' ||
    'For each SELECT statement that you write, the data it returns will be analyzed and rendered to the user.';
select 'The two most important concepts in SQLPage are ' as contents;
select 'components' as contents, true as bold;
select ' and ' as contents;
select 'parameters' as contents, true as bold;
select '.' as contents;
select 'This page documents all the components that you can use in SQLPage and their parameters. ' ||
     'Use this as a reference when building your SQL application.' as contents;

select 'list' as component, 'components' as title;
select
    name as title,
    description,
    icon,
    '?component='||name||'#component' as link,
    $1->>'$.query.component' = name as active
from component;

select 'text' as component,
    'The "'||($1->>'$.query.component')||'" component' as title,
    'component' as id;
select description as contents from component where name = $1->>'$.query.component';

select 'title' as component, 3 as level, 'Parameters' as contents where $1->>'$.query.component' IS NOT NULL;
select 'card' as component, 3 AS columns where $1->>'$.query.component' IS NOT NULL;
select
    name || (CASE WHEN top_level THEN ' (top-level)' ELSE '' END) as title,
    (CASE WHEN optional THEN '' ELSE 'REQUIRED. ' END) || description as body,
    type as footer,
    CASE WHEN top_level THEN 'lime' ELSE 'azure' END || CASE WHEN optional THEN '-lt' ELSE '' END as color
from parameter where component = $1->>'$.query.component'
ORDER BY (NOT top_level), optional, name;

select
    'dynamic' as component,
    json_array(
        json_object('component', 'code'),
        json_object(
            'title', 'Example',
            'description', description,
            'contents', (
                select
                     group_concat(
                        'SELECT ' || x'0A' ||
                            (
                                select group_concat(
                                    '    ' || quote(value) || ' as ' || key, ',' || x'0A'
                                ) from json_each(top.value)
                            ) || ';',
                        x'0A'
                     )
                from json_each(properties) top
            )
        ),
        json_object('component', 'title', 'level', 3, 'contents', 'Result'),
        json_object('component', 'dynamic', 'properties', properties)
    ) as properties
from example where component = $1->>'$.query.component';