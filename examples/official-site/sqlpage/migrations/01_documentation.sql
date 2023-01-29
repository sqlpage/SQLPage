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

INSERT INTO example(component, description, properties) VALUES
    ('list', 'The most basic list', json('[{"component":"list"},{"title":"A"},{"title":"B"},{"title":"C"}]')),
    ('list', 'A beautiful list with bells and whistles.',
            json('[{"component":"list", "title":"Popular websites"}, '||
            '{"title":"Google", "link":"https://google.com", "description": "A search engine", "color": "red", "icon":"brand-google", "active": true}, '||
            '{"title":"Wikipedia", "link":"https://wikipedia.org", "description": "An encyclopedia", "color": "blue", "icon":"world"}]'));

INSERT INTO component(name, icon, description) VALUES
    ('card', 'credit-card', 'A grid where each element is a small card that displays a piece of data.');
INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'card', * FROM (VALUES
    -- top level
    ('title', 'Text header at the top of the list of cards.', 'TEXT', TRUE, TRUE),
    ('columns', 'The number of columns in the grid of cards. This is just a hint, the grid will adjust dynamically to the user''s screen size, rendering fewer columns if needed to fit the contents.', 'INTEGER', TRUE, TRUE),
    -- item level
    ('title', 'Name of the card, displayed at the top.', 'TEXT', FALSE, FALSE),
    ('description', 'The body of the card.', 'TEXT', FALSE, TRUE),
    ('footer', 'Muted text to display at the bottom of the card.', 'TEXT', FALSE, TRUE),
    ('link', 'An URL to which the user should be taken when they click on the card.', 'URL', FALSE, TRUE),
    ('icon', 'An icon name (from tabler-icons.io) to display on the left side of the card.', 'TEXT', FALSE, TRUE),
    ('color', 'The name of a color, to be displayed on the left of the card to highlight it.', 'TEXT', FALSE, TRUE),
    ('active', 'Whether this item in the grid is considered "active". Active items are displayed more prominently.', 'BOOLEAN', FALSE, TRUE)
);

INSERT INTO example(component, description, properties) VALUES
    ('card', 'The most basic card', json('[{"component":"card"},{"title":"A"},{"title":"B"},{"title":"C"}]')),
    ('card', 'A beautiful card grid with bells and whistles.',
            json('[{"component":"card", "title":"Popular websites", "columns": 2}, '||
            '{"title":"Google", "link":"https://google.com", "description": "A search engine", "color": "red", "icon":"brand-google", "footer": "Owned by Alphabet Inc."}, '||
            '{"title":"Wikipedia", "link":"https://wikipedia.org", "description": "An encyclopedia", "color": "blue", "icon":"world", "active": true, "footer": "Owned by the Wikimedia Foundation"}]'));


INSERT INTO component(name, icon, description) VALUES
    ('datagrid', 'grid-dots', 'Display small pieces of information in a clear and readable way. Each item has a name and is associated with a value.');
INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'datagrid', * FROM (VALUES
    -- top level
    ('title', 'Text header at the top of the data grid.', 'TEXT', TRUE, TRUE),
    -- item level
    ('title', 'Name of the piece of information.', 'TEXT', FALSE, FALSE),
    ('description', 'Value to display below the name.', 'TEXT', FALSE, TRUE),
    ('footer', 'Muted text to display below the value.', 'TEXT', FALSE, TRUE),
    ('link', 'A target URL to which the user should be taken when they click on the value.', 'URL', FALSE, TRUE),
    ('icon', 'An icon name (from tabler-icons.io) to display on the left side of the value.', 'TEXT', FALSE, TRUE),
    ('color', 'If set to a color name, the value will be displayed in a pill of that color.', 'TEXT', FALSE, TRUE),
    ('active', 'Whether this item in the grid is considered "active". Active items are displayed more prominently.', 'BOOLEAN', FALSE, TRUE)
);

INSERT INTO example(component, description, properties) VALUES
    ('datagrid', 'Just some sections of information.', json('[{"component":"datagrid"},{"title":"Language","description":"SQL"},{"title":"Creation date","description":"1974"}, {"title":"Language family","description":"Query language"}]')),
    ('datagrid', 'A beautiful data grid with nice colors and icons.',
            json('[{"component":"datagrid", "title":"User"}, '||
            '{"title": "Pseudo", "description": "lovasoa"},' ||
            '{"title": "Status", "description": "Active", "color": "green"},' ||
            '{"title": "Email Status", "description": "Validated", "icon": "check", "active": true},' ||
            '{"title": "Personal page", "description": "ophir.dev", "link": "https://ophir.dev/"},' ||
            '{"title":"Search engine", "link":"https://google.com", "description": "Google", "color": "red", "icon":"brand-google", "footer": "Owned by Alphabet Inc."}, '||
            '{"title":"Encyclopedia", "link":"https://wikipedia.org", "description": "Wikipedia", "color": "blue", "icon":"world", "footer": "Owned by the Wikimedia Foundation"}]'));


INSERT INTO component(name, icon, description) VALUES
    ('steps', 'dots-circle-horizontal', 'Guide users through multi-stage processes, displaying a clear list of previous and future steps.');
INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'steps', * FROM (VALUES
    -- top level
    ('color', 'Color of the bars displayed between steps.', 'TEXT', TRUE, TRUE),
    ('counter', 'Display the number of the step on top of its name.', 'TEXT', TRUE, TRUE),
    -- item level
    ('title', 'Name of the step.', 'TEXT', FALSE, TRUE),
    ('description', 'Tooltip to display when the user passes their mouse over the step''s name.', 'TEXT', FALSE, TRUE),
    ('link', 'A target URL to which the user should be taken when they click on the step.', 'URL', FALSE, TRUE),
    ('icon', 'An icon name (from tabler-icons.io) to display on the left side of the step name.', 'TEXT', FALSE, TRUE),
    ('active', 'Whether this item in the grid is considered "active". Active items are displayed more prominently.', 'BOOLEAN', FALSE, TRUE)
);

INSERT INTO example(component, description, properties) VALUES
    ('steps', 'Online store checkout steps.', json('[{"component":"steps"},{"title":"Shopping"},{"title":"Store pickup"}, {"title":"Payment","active":true},{"title":"Review & Order"}]')),
    ('steps', 'A progress indicator with custom color, auto-generated step numbers, icons, and description tooltips.',
            json('[{"component":"steps", "counter": true, "color":"purple"}, '||
            '{"title": "Registration form", "icon":"forms", "link": "https://github.com/lovasoa/sqlpage", "description": "Initial account data creation."},' ||
            '{"title": "Email confirmation", "icon": "mail", "link": "https://sql.ophir.dev", "description": "Confirm your email by clicking on a link in a validation email."},' ||
            '{"title": "ID verification", "description": "Checking personal information", "icon": "user", "link": "#"},' ||
            '{"title": "Final account approval", "description": "ophir.dev", "link": "https://ophir.dev/", "icon":"eye-check", "active": true},' ||
            '{"title":"Account creation", "icon":"check"}]'));

INSERT INTO component(name, icon, description) VALUES
    ('text', 'align-left', 'A paragraph of text. The entire component will render as a single paragraph, with each item being rendered as a span of text inside it, the styling of which can be customized using parameters.');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'text', * FROM (VALUES
    -- top level
    ('title', 'Text header before the paragraph.', 'TEXT', TRUE, TRUE),
    ('center', 'Whether to center the title.', 'BOOLEAN', TRUE, TRUE),
    ('width', 'How wide the paragraph should be, in characters.', 'INTEGER', TRUE, TRUE),
    ('html', 'Raw html code to include on the page. Don''t use that if you are not sure what you are doing, it may have security implications.', 'TEXT', TRUE, TRUE),
    -- item level
    ('contents', 'A span of text to display', 'TEXT', FALSE, FALSE),
    ('link', 'An URL to which the user should be taken when they click on this span of text.', 'URL', FALSE, TRUE),
    ('color', 'The name of a color for this span of text.', 'TEXT', FALSE, TRUE),
    ('underline', 'Whether the span of text should be underlined.', 'BOOLEAN', FALSE, TRUE),
    ('bold', 'Whether the span of text should be displayed as bold.', 'BOOLEAN', FALSE, TRUE),
    ('code', 'Use a monospace font. Useful to display the text as code.', 'BOOLEAN', FALSE, TRUE),
    ('italics', 'Whether the span of text should be displayed as italics.', 'BOOLEAN', FALSE, TRUE),
    ('break', 'Indicates that the current span of text starts a new paragraph.', 'BOOLEAN', FALSE, TRUE),
    ('size', 'A number between 1 and 6 indicating the font size.', 'INTEGER', FALSE, TRUE)
);

INSERT INTO example(component, description, properties) VALUES
    ('text', 'Rendering a simple text paragraph.', json('[{"component":"text", "contents":"Hello, world ! <3"}]')),
    ('text', 'Rendering a paragraph with links and styling.',
            json('[{"component":"text", "title":"About SQL"}, '||
            '{"contents":"SQL", "bold":true, "italics": true}, '||
            '{"contents":" is a domain-specific language used in programming and designed for managing data held in a "},'||
            '{"contents": "relational database management system", "link": "https://en.wikipedia.org/wiki/Relational_database"},'||
            '{"contents": ". It is particularly useful in handling structured data."}]')
);

INSERT INTO component(name, icon, description) VALUES
    ('form', 'cursor-text', 'A series of input fields that can be filled in by the user. ' ||
    'The form contents can be posted and handled by another SQLPage. ' ||
    'The value entered by the user in a field named x will be accessible to the target SQL page as $x.');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'form', * FROM (VALUES
    -- top level
    ('method', 'Set this to ''GET'' to pass the form contents directly as URL parameters. If the user enters a value v in a field named x, submitting the form will load target.sql?x=v. If target.sql contains SELECT $x, it will display the value v.', 'TEXT', TRUE, TRUE),
    ('action', 'An optional link to a target page that will handle the results of the form. By default the target page is the current page. Setting it to the name of a different sql file will load that file when the user submits the form.', 'TEXT', TRUE, TRUE),
    ('title', 'A name to display at the top of the form. It will be displayed in a larger font size at the top of the form.', 'TEXT', TRUE, TRUE),
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
INSERT INTO example(component, description, properties) VALUES
    ('form', 'A user registration form.', json('[{"component":"form", "title": "User", "validate": "Create new user"}, '||
    '{"name": "First name", "placeholder": "John"}, '||
    '{"name": "Last name", "required": true, "description": "We need your last name for legal purposes."},'||
    '{"name": "Birth date", "type": "date", "max": "2010-01-01"}]'));

INSERT INTO component(name, icon, description) VALUES
    ('chart', 'timeline', 'A component that plots data. Line, area, bar, and pie charts are all supported. Each item in the component is a data point in the graph.');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'chart', * FROM (VALUES
    -- top level
    ('title', 'The name of the chart.', 'TEXT', TRUE, TRUE),
    ('type', 'The type of chart: "line", "area", "bar", "column", or "pie".', 'TEXT', TRUE, FALSE),
    ('time', 'Whether the x-axis represents time. If set to true, the values will be formatted as dates for the user.', 'BOOLEAN', TRUE, TRUE),
    ('ymin', 'The minimal value for the y-axis.', 'NUMBER', TRUE, TRUE),
    ('ymax', 'The maximum value for the y-axis.', 'NUMBER', TRUE, TRUE),
    ('labels', 'Whether to show the data labels on the chart or not.', 'BOOLEAN', TRUE, TRUE),
    ('stacked', 'Whether to cumulate values from different series.', 'BOOLEAN', TRUE, TRUE),
    ('logarithmic', 'Display the y-axis in logarithmic scale..', 'BOOLEAN', TRUE, TRUE),
    -- item level
    ('x', 'The value of the point on the horizontal axis', 'NUMBER', FALSE, FALSE),
    ('y', 'The value of the point on the vertical axis', 'NUMBER', FALSE, FALSE),
    ('label', 'An alias for parameter "x"', 'NUMBER', FALSE, TRUE),
    ('value', 'An alias for parameter "y"', 'NUMBER', FALSE, TRUE),
    ('series', 'If multiple series are represented and share the same y-axis, this parameter can be used to distinguish between them.', 'TEXT', FALSE, TRUE)
);
INSERT INTO example(component, description, properties) VALUES
    ('chart', 'A pie chart.', json('[{"component":"chart", "title": "Answers", "type": "pie", "labels": true}, '||
    '{"label": "Yes", "value": 65}, '||
    '{"label": "No", "value": 35}]')),
    ('chart', 'An area chart', json('[{"component":"chart", "title": "Syracuse", "type": "area"}, '||
    '{"x":0,"y":15},{"x":1,"y":46},{"x":2,"y":23},{"x":3,"y":70},{"x":4,"y":35},{"x":5,"y":106}]')),
    ('chart', 'A bar chart with multiple series.', json('[{"component":"chart", "title": "Expenses", "type": "bar", "stacked": true}, '||
    '{"series": "Marketing", "x": 2021, "value": 35}, '||
    '{"series": "Marketing", "x": 2022, "value": 15}, '||
    '{"series": "Human resources", "x": 2021, "value": 30}, '||
    '{"series": "Human resources", "x": 2022, "value": 55}]'));

INSERT INTO component(name, icon, description) VALUES
    ('table', 'table', 'A table with optional filtering and sorting. Unlike most others, this component does not have a fixed set of item properties, any property that is used will be rendered directly as a column in the table.');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'table', * FROM (VALUES
    -- top level
    ('sort', 'Make the columns clickable to let the user sort by the value contained in the column.', 'BOOLEAN', TRUE, TRUE),
    ('search', 'Add a search bar at the top of the table, letting users easily filter table rows by value.', 'BOOLEAN', TRUE, TRUE)
);

INSERT INTO example(component, description, properties) VALUES
    ('table', 'A table of users with filtering and sorting.',
        json('[{"component":"table", "sort":true, "search":true}, '||
        '{"Forename": "Ophir", "Surname": "Lojkine", "Pseudonym": "lovasoa"},' ||
        '{"Forename": "Linus", "Surname": "Torvalds", "Pseudonym": "torvalds"}]'));


INSERT INTO component(name, icon, description) VALUES
    ('csv', 'download', 'A button that lets the user download data as a CSV file. Each column from the items in the component will map to a column in the resulting CSV.');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'csv', * FROM (VALUES
    -- top level
    ('separator', 'How individual values should be separated in the CSV. "," by default, set it to "\t" for tab-separated values.', 'TEXT', TRUE, TRUE),
    ('title', 'The text displayed on the download button.', 'TEXT', TRUE, FALSE),
    ('filename', 'The name of the file that should be downloaded (without the extension).', 'TEXT', TRUE, TRUE),
    ('icon', 'Name of the icon (from tabler-icons.io) to display in the button.', 'TEXT', TRUE, TRUE),
    ('color', 'Color of the button', 'TEXT', TRUE, TRUE)
);

INSERT INTO example(component, description, properties) VALUES
    ('csv', 'CSV download button',
        json('[{"component":"csv", "title": "Download my data", "filename": "people", "icon": "file-download", "color": "green"}, '||
        '{"Forename": "Ophir", "Surname": "Lojkine", "Pseudonym": "lovasoa"},' ||
        '{"Forename": "Linus", "Surname": "Torvalds", "Pseudonym": "torvalds"}]'));


INSERT INTO component(name, icon, description) VALUES
    ('dynamic', 'repeat', 'A special component that can be used to render other components, the number and properties of which are not known in advance.');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'dynamic', * FROM (VALUES
    -- top level
    ('properties', 'A json object or array that contains the names and properties of other components', 'JSON', TRUE, TRUE)
);

INSERT INTO example(component, description, properties) VALUES
    ('dynamic', 'Rendering a text paragraph dynamically.', json('[{"component":"dynamic", "properties": "[{\"component\":\"text\"}, {\"contents\":\"Blah\", \"bold\":true}]"}]'));

INSERT INTO component(name, icon, description) VALUES
    ('shell', 'layout-navbar', 'Personalize the "shell" surrounding your page contents. Used to set properties for the entire page.');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'shell', * FROM (VALUES
    -- top level
    ('title', 'The title of your page. Will be shown in a top bar above the page contents. Also usually displayed by web browsers as the name of the web page''s tab.', 'TEXT', TRUE, TRUE),
    ('description', 'A description of the page. It can be displayed by search engines when your page appears in their results.', 'TEXT', TRUE, TRUE),
    ('link', 'The target of the link in the top navigation bar.', 'URL', TRUE, TRUE),
    ('image', 'The URL of an image to display next to the page title.', 'URL', TRUE, TRUE),
    ('icon', 'Name of an icon (from tabler-icons.io) to display next to the title in the navigation bar.', 'TEXT', TRUE, TRUE),
    ('menu_item', 'Adds a menu item in the navigation bar at the top of the page. The menu item will have the specified name, and will link to as .sql file of the same name.', 'TEXT', TRUE, TRUE),
    ('search_target', 'When this is set, a search field will appear in the top navigation bar, and load the specified sql file with an URL parameter named "search" when the user searches something.', 'TEXT', TRUE, TRUE),
    ('norobot', 'Forbids robots to save this page in their database and follow the links on this page. This will prevent this page to appear in Google search results for any query, for instance.', 'BOOLEAN', TRUE, TRUE)
);

INSERT INTO example(component, description, properties) VALUES
    ('shell', 'This example contains the values used for the shell of the page you are currently viewing.',
     json('[{"title": "SQLPage documentation", "link": "/", "lang": "en-US", "description": "Documentation for the SQLPage low-code web application framework."}]'));
