CREATE TABLE component(
    name TEXT PRIMARY KEY,
    description TEXT NOT NULL,
    icon TEXT, -- icon name from tabler icon
    introduced_in_version TEXT
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
    properties JSON,
    FOREIGN KEY (component) REFERENCES component(name) ON DELETE CASCADE
);

INSERT INTO component(name, icon, description) VALUES
    ('list', 'list', 'A vertical list of items. Each item can be clickable and link to another page.');
INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'list', * FROM (VALUES
    -- top level
    ('title', 'Text header at the top of the list.', 'TEXT', TRUE, TRUE),
    ('empty_title', 'Title text to display if the list is empty.', 'TEXT', TRUE, TRUE),
    ('empty_description', 'Description to display if the list is empty.', 'TEXT', TRUE, TRUE),
    ('empty_description_md', 'Description to display if the list is empty, in Markdown format.', 'TEXT', TRUE, TRUE),
    ('empty_link', 'URL to which the user should be taken if they click on the empty list.', 'URL', TRUE, TRUE),
    -- item level
    ('title', 'Name of the list item, displayed prominently.', 'TEXT', FALSE, FALSE),
    ('description', 'A description of the list item, displayed as greyed-out text.', 'TEXT', FALSE, TRUE),
    ('description_md', 'A description of the list item, displayed as greyed-out text, in Markdown format, allowing you to use rich text formatting, including **bold** and *italic* text.', 'TEXT', FALSE, TRUE),
    ('link', 'An URL to which the user should be taken when they click on the list item.', 'URL', FALSE, TRUE),
    ('icon', 'Name of an icon to display on the left side of the item.', 'ICON', FALSE, TRUE),
    ('color', 'The name of a color, to be displayed as a dot near the list item contents.', 'COLOR', FALSE, TRUE),
    ('active', 'Whether this item in the list is considered "active". Active items are displayed more prominently.', 'BOOLEAN', FALSE, TRUE)
) x;

INSERT INTO example(component, description, properties) VALUES
    ('list', 'The most basic list', json('[{"component":"list"},{"title":"A"},{"title":"B"},{"title":"C"}]')),
    ('list', 'An empty list with a link to add an item', json('[{"component":"list", "empty_title": "No items yet", "empty_description": "This list is empty. Click here to create a new item !", "empty_link": "documentation.sql"}]')),
    ('list', 'A list with rich text descriptions', json('[{"component":"list"},
        {"title":"SQLPage", "description_md":"A **SQL**-based **page** generator for **PostgreSQL**, **MySQL**, and **SQLite**. [Free on Github](https://github.com/lovasoa/sqlpage)"},
        {"title":"Tabler", "description_md":"A **free** and **open-source** **HTML** template pack based on **Bootstrap**."},
        {"title":"Tabler Icons", "description_md":"A set of over **700** free MIT-licensed high-quality **SVG** icons for you to use in your web projects."}
    ]')),
    ('list', 'A beautiful list with bells and whistles.',
            json('[{"component":"list", "title":"Popular websites"}, '||
            '{"title":"Google", "link":"https://google.com", "description": "A search engine", "color": "red", "icon":"brand-google", "active": true}, '||
            '{"title":"Wikipedia", "link":"https://wikipedia.org", "description": "An encyclopedia", "color": "blue", "icon":"world"}]'));

INSERT INTO component(name, icon, description) VALUES
    ('card', 'credit-card', 'A grid where each element is a small card that displays a piece of data.');
INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'card', * FROM (VALUES
    -- top level
    ('title', 'Text header at the top of the list of cards.', 'TEXT', TRUE, TRUE),
    ('description', 'A short paragraph displayed below the title.', 'TEXT', TRUE, TRUE),
    ('description_md', 'A short paragraph displayed below the title - formatted using markdown.', 'TEXT', TRUE, TRUE),
    ('columns', 'The number of columns in the grid of cards. This is just a hint, the grid will adjust dynamically to the user''s screen size, rendering fewer columns if needed to fit the contents.', 'INTEGER', TRUE, TRUE),
    -- item level
    ('title', 'Name of the card, displayed at the top.', 'TEXT', FALSE, FALSE),
    ('description', 'The body of the card, where you put the main text contents of the card.
        This does not support rich text formatting, only plain text.
        If you want to use rich text formatting, use the `description_md` property instead.', 'TEXT', FALSE, TRUE),
    ('description_md', '
        The body of the card, in Markdown format.
        This is useful if you want to display a lot of text in the card, with many options for formatting, such as
        line breaks, **bold**, *italics*, lists, #titles, [links](target.sql), ![images](photo.jpg), etc.', 'TEXT', FALSE, TRUE),
    ('top_image', 'The URL (absolute or relative) of an image to display at the top of the card.', 'URL', FALSE, TRUE),
    ('footer', 'Muted text to display at the bottom of the card.', 'TEXT', FALSE, TRUE),
    ('footer_md', 'Muted text to display at the bottom of the card, with rich text formatting in Markdown format.', 'TEXT', FALSE, TRUE),
    ('link', 'An URL to which the user should be taken when they click on the card.', 'URL', FALSE, TRUE),
    ('footer_link', 'An URL to which the user should be taken when they click on the footer.', 'URL', FALSE, TRUE),
    ('icon', 'Name of an icon to display on the left side of the card.', 'ICON', FALSE, TRUE),
    ('color', 'The name of a color, to be displayed on the left of the card to highlight it.', 'COLOR', FALSE, TRUE),
    ('active', 'Whether this item in the grid is considered "active". Active items are displayed more prominently.', 'BOOLEAN', FALSE, TRUE)
) x;

INSERT INTO example(component, description, properties) VALUES
    ('card', 'The most basic card', json('[{"component":"card"},{"title":"A"},{"title":"B"},{"title":"C"}]')),
    ('card', 'A card with a Markdown description',
            json('[{"component":"card"}, {"title":"A card with a Markdown description", "description_md": "This is a card with a **Markdown** description. \n\n'||
            'This is useful if you want to display a lot of text in the card, with many options for formatting, such as '||
            '\n - **bold**, \n - *italics*, \n - [links](index.sql), \n - etc."}]')),
    ('card', 'A beautiful card grid with bells and whistles.',
            json('[{"component":"card", "title":"Popular websites", "columns": 2}, '||
            '{"title":"Google", "link":"https://google.com", "description": "A search engine", "color": "red", "icon":"brand-google", "footer": "Owned by Alphabet Inc.", "footer_link": "https://abc.xyz/" }, '||
            '{"title":"Wikipedia", "link":"https://wikipedia.org", "description": "An encyclopedia", "color": "blue", "icon":"world", "active": true, "footer": "Owned by the Wikimedia Foundation", "footer_link": "https://wikimediafoundation.org/"}]')),
    ('card', 'A gallery of images.',
        json('[
            {"component":"card", "title":"My favorite animals in pictures"},
            {"title": "Lynx", "description_md": "The **lynx** is a medium-sized **wild cat** native to Northern, Central and Eastern Europe to Central Asia and Siberia, the Tibetan Plateau and the Himalayas.", "top_image": "https://upload.wikimedia.org/wikipedia/commons/thumb/d/d8/Lynx_lynx-4.JPG/640px-Lynx_lynx-4.JPG", "icon":"star" },
            {"title": "Squirrel", "description_md": "The **chipmunk** is a small, striped rodent of the family Sciuridae. Chipmunks are found in North America, with the exception of the Siberian chipmunk which is found primarily in Asia.", "top_image": "https://upload.wikimedia.org/wikipedia/commons/thumb/b/be/Tamias-rufus-001.jpg/640px-Tamias-rufus-001.jpg" },
            {"title": "Spider", "description_md": "The **jumping spider family** (_Salticidae_) contains more than 600 described genera and about *6000 described species*, making it the largest family of spiders with about 13% of all species.", "top_image": "https://upload.wikimedia.org/wikipedia/commons/thumb/a/ab/Jumping_spiders_%28Salticidae%29.jpg/640px-Jumping_spiders_%28Salticidae%29.jpg" }
        ]'));


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
    ('icon', 'An icon name (from tabler-icons.io) to display on the left side of the value.', 'ICON', FALSE, TRUE),
    ('color', 'If set to a color name, the value will be displayed in a pill of that color.', 'COLOR', FALSE, TRUE),
    ('active', 'Whether this item in the grid is considered "active". Active items are displayed more prominently.', 'BOOLEAN', FALSE, TRUE)
) x;

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
    ('color', 'Color of the bars displayed between steps.', 'COLOR', TRUE, TRUE),
    ('counter', 'Display the number of the step on top of its name.', 'TEXT', TRUE, TRUE),
    ('title', 'Title of the section.', 'TEXT', TRUE, TRUE),
    ('description', 'Description of the section.', 'TEXT', TRUE, TRUE),
    -- item level
    ('title', 'Name of the step.', 'TEXT', FALSE, TRUE),
    ('description', 'Tooltip to display when the user passes their mouse over the step''s name.', 'TEXT', FALSE, TRUE),
    ('link', 'A target URL to which the user should be taken when they click on the step.', 'URL', FALSE, TRUE),
    ('icon', 'An icon name (from tabler-icons.io) to display on the left side of the step name.', 'ICON', FALSE, TRUE),
    ('active', 'Whether this item in the grid is considered "active". Active items are displayed more prominently.', 'BOOLEAN', FALSE, TRUE)
) x;

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
    ('contents', 'A top-level paragraph of text to display, without any formatting, without having to make additional queries.', 'TEXT', TRUE, TRUE),
    ('contents_md', 'Rich text in the markdown format. Among others, this allows you to write bold text using **bold**, italics using *italics*, and links using [text](https://example.com).', 'TEXT', TRUE, TRUE),
    -- item level
    ('contents', 'A span of text to display', 'TEXT', FALSE, FALSE),
    ('link', 'An URL to which the user should be taken when they click on this span of text.', 'URL', FALSE, TRUE),
    ('color', 'The name of a color for this span of text.', 'COLOR', FALSE, TRUE),
    ('underline', 'Whether the span of text should be underlined.', 'BOOLEAN', FALSE, TRUE),
    ('bold', 'Whether the span of text should be displayed as bold.', 'BOOLEAN', FALSE, TRUE),
    ('code', 'Use a monospace font. Useful to display the text as code.', 'BOOLEAN', FALSE, TRUE),
    ('italics', 'Whether the span of text should be displayed as italics.', 'BOOLEAN', FALSE, TRUE),
    ('break', 'Indicates that the current span of text starts a new paragraph.', 'BOOLEAN', FALSE, TRUE),
    ('size', 'A number between 1 and 6 indicating the font size.', 'INTEGER', FALSE, TRUE)
) x;

INSERT INTO example(component, description, properties) VALUES
    ('text', 'Rendering a simple text paragraph.', json('[{"component":"text", "contents":"Hello, world ! <3"}]')),
    ('text', 'Rendering rich text using markdown', json('[{"component":"text", "contents_md":"\n'||
    '# Markdown in SQLPage\n\n' ||
    '## Simple formatting\n\n' ||
    'SQLPage supports only plain text as column values, but markdown allows easily adding **bold**, *italics*, and [links](index.sql).\n\n' ||
    '## Lists\n' ||
    '### Unordered lists\n' ||
    '* SQLPage is easy\n' ||
    '* SQLPage is fun\n' ||
    '* SQLPage is free\n\n' ||
    '### Ordered lists\n' ||
    '1. SQLPage is fast\n' ||
    '2. SQLPage is safe\n' ||
    '3. SQLPage is open-source\n\n' ||
    '## Code\n' ||
    '```sql\n' ||
    'SELECT ''list'' AS component;\n' ||
    'SELECT name as title FROM users;\n' ||
    '```\n\n' ||
    '## Tables\n\n' ||
    '| SQLPage component | Description  | Documentation link  |\n' ||
    '| --- | --- | --- |\n' ||
    '| text | A paragraph of text. | [Documentation](https://sql.ophir.dev/documentation.sql?component=text) |\n' ||
    '| list | A list of items. | [Documentation](https://sql.ophir.dev/documentation.sql?component=list) |\n' ||
    '| steps | A progress indicator. | [Documentation](https://sql.ophir.dev/documentation.sql?component=steps) |\n' ||
    '| form | A series of input fields. | [Documentation](https://sql.ophir.dev/documentation.sql?component=form) |\n\n' ||
    '## Quotes\n' ||
    '> Fantastic.\n>\n' ||
    '> — [HackerNews User](https://news.ycombinator.com/item?id=36194473#36209061) about SQLPage\n\n' ||
    '## Images\n' ||
    '![SQLPage logo](https://sql.ophir.dev/favicon.ico)\n\n' ||
    '## Horizontal rules\n' ||
    '---\n\n' ||
    '"}]')),
    ('text', 'Rendering a paragraph with links and styling.',
            json('[{"component":"text", "title":"About SQL"}, '||
            '{"contents":"SQL", "bold":true, "italics": true}, '||
            '{"contents":" is a domain-specific language used in programming and designed for managing data held in a "},'||
            '{"contents": "relational database management system", "link": "https://en.wikipedia.org/wiki/Relational_database"},'||
            '{"contents": ". It is particularly useful in handling structured data."}]')
);

INSERT INTO component(name, icon, description) VALUES
    ('form', 'cursor-text', 'A series of input fields that can be filled in by the user. ' ||
    'The form contents can be posted and handled by another sql file in your site. ' ||
    'The value entered by the user in a field named x will be accessible to the target SQL page as a variable named $x.
    For instance, you can create a SQL page named "create_user.sql" that would contain "INSERT INTO users(name) VALUES($name)"
    and a form with its action property set to "create_user.sql" that would contain a field named "name".');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'form', * FROM (VALUES
    -- top level
    ('method', 'Set this to ''GET'' to pass the form contents directly as URL parameters. If the user enters a value v in a field named x, submitting the form will load target.sql?x=v. If target.sql contains SELECT $x, it will display the value v.', 'TEXT', TRUE, TRUE),
    ('action', 'An optional link to a target page that will handle the results of the form. By default the target page is the current page. Setting it to the name of a different sql file will load that file when the user submits the form.', 'TEXT', TRUE, TRUE),
    ('title', 'A name to display at the top of the form. It will be displayed in a larger font size at the top of the form.', 'TEXT', TRUE, TRUE),
    ('validate', 'The text to display in the button at the bottom of the form that submits the values. Omit this property to let the browser display the default form validation text, or set it to the empty string to remove the button completely.', 'TEXT', TRUE, TRUE),
    ('validate_color', 'The color of the button at the bottom of the form that submits the values. Omit this property to use the default color.', 'COLOR', TRUE, TRUE),
    ('validate_outline', 'A color to outline the validation button.', 'COLOR', TRUE, TRUE),
    ('reset', 'The text to display in the button at the bottom of the form that resets the form to its original state. Omit this property not to show a reset button at all.', 'TEXT', TRUE, TRUE),
    ('id', 'A unique identifier for the form, which can then be used to validate the form from a button outside of the form.', 'TEXT', TRUE, TRUE),
    -- item level
    ('type', 'The type of input to use: text for a simple text field, textarea for a multi-line text input control, number for field that accepts only numbers, checkbox or radio for a button that is part of a group specified in the ''name'' parameter. This is set to "text" by default.', 'TEXT', FALSE, TRUE),
    ('name', 'The name of the input field, that you can use in the target page to get the value the user entered for the field.', 'TEXT', FALSE, FALSE),
    ('label', 'A friendly name for the text field to show to the user.', 'TEXT', FALSE, TRUE),
    ('placeholder', 'A placeholder text that will be shown in the field when is is empty.', 'TEXT', FALSE, TRUE),
    ('value', 'A default value that will already be present in the field when the user loads the page.', 'TEXT', FALSE, TRUE),
    ('options', 'A json array of objects containing the label and value of all possible options of a select field. Used only when type=select.', 'JSON', FALSE, TRUE),
    ('required', 'Set this to true to prevent the form contents from being sent if this field is left empty by the user.', 'BOOL', FALSE, TRUE),
    ('min', 'The minimum value to accept for an input of type number', 'NUMBER', FALSE, TRUE),
    ('max', 'The minimum value to accept for an input of type number', 'NUMBER', FALSE, TRUE),
    ('checked', 'Used only for checkboxes and radio buttons. Indicates whether the checkbox should appear as already checked.', 'BOOL', FALSE, TRUE),
    ('multiple', 'Used only for select elements. Indicates that multiple elements can be selected simultaneously. When using multiple, you should add square brackets after the variable name: ''my_variable[]'' as name', 'BOOL', FALSE, TRUE),
    ('step', 'The increment of values in an input of type number. Set to 1 to allow only integers.', 'NUMBER', FALSE, TRUE),
    ('description', 'A helper text to display near the input field.', 'TEXT', FALSE, TRUE),
    ('pattern', 'A regular expression that the value must match. For instance, [0-9]{3} will only accept 3 digits.', 'TEXT', FALSE, TRUE),
    ('autofocus', 'Automatically focus the field when the page is loaded', 'BOOL', FALSE, TRUE),
    ('width', 'Width of the form field, between 1 and 12.', 'NUMBER', FALSE, TRUE),
    ('autocomplete', 'Whether the browser should suggest previously entered values for this field.', 'BOOL', FALSE, TRUE),
    ('minlength', 'Minimum length of text allowed in the field.', 'NUMBER', FALSE, TRUE),
    ('maxlength', 'Maximum length of text allowed in the field.', 'NUMBER', FALSE, TRUE),
    ('formaction', 'When type is "submit", this specifies the URL of the file that will handle the form submission. Useful when you need multiple submit buttons.', 'TEXT', FALSE, TRUE),
    ('class', 'A CSS class to apply to the form element.', 'TEXT', FALSE, TRUE)
) x;
INSERT INTO example(component, description, properties) VALUES
    (
    'form',
    '

The best way to manage forms in SQLPage is to create at least two separate files:

 - one that will contain the form itself, and will be loaded when the user visits the page,
 - one that will handle the form submission, and will redirect to whatever page you want to display after the form has been submitted.

For instance, if you were creating a form to manage a list of users, you could create:

 - a file named `users.sql` that would contain a list of users and a form to create a new user,
 - a file named `create_user.sql` that would insert the new user in the database, and then redirect to `users.sql`.

`create_user.sql` could contain the following sql statement to [safely](safety.sql) insert the new user in the database: 

```sql
INSERT INTO users(name) VALUES(:username)
RETURNING ''redirect'' AS component, ''users.sql'' AS link
```

When loading the page, the value for `:username` will be `NULL` if no value has been submitted.
',
    json('[{"component":"form", "action": "create_user.sql"}, {"name": "username"}]')),
    ('form', 'A user registration form, illustrating the use of required fields, and different input types.', 
    json('[{"component":"form", "title": "User", "validate": "Create new user"}, '||
    '{"name": "First name", "placeholder": "John"}, '||
    '{"name": "Last name", "required": true, "description": "We need your last name for legal purposes."},'||
    '{"name": "Resume", "type": "textarea"},'||
    '{"name": "Birth date", "type": "date", "max": "2010-01-01", "value": "1994-04-16"},'||
    '{"name": "Password", "type": "password", "pattern": "^(?=.*[A-Za-z])(?=.*\\d)[A-Za-z\\d]{8,}$", "required": true, "description": "Minimum eight characters, at least one letter and one number."},'||
    '{"label": "I accept the terms and conditions", "name": "terms", "type": "checkbox", "required": true}'||
    ']')),
    ('form', 'This example illustrates the use of the `select` type.
In this select input, the various options are hardcoded, but they could also be loaded from a database table,
using a function to convert the rows into a json array like 
 - `json_group_array()` in SQLite,
 - `json_agg()` in Postgres,
 - `JSON_ARRAYAGG()` in MySQL, or
 - `FOR JSON PATH` in Microsoft SQL Server.


In SQLite, the query would look like
```sql
SELECT 
    ''select'' as type,
    json_group_array(json_object(
        "label", name,
        "value", id
    )) as options
FROM fruits
```
', json('[{"component":"form"}, '||
    '{"name": "Fruit", "type": "select", "value": 1, "options": '||
        '"[{\"label\": \"Orange\", \"value\": 0}, {\"label\": \"Apple\", \"value\": 1}, {\"label\": \"Banana\", \"value\": 3}]"}
    ]')),
    ('form', '### Multi-select
You can authorize the user to select multiple options by setting the `multiple` property to `true`.
This creates a more compact (but arguably less user-friendly) alternative to a series of checkboxes.
In this case, you should add square brackets to the name of the field.
The target page will then receive the value as a JSON array of strings, which you can iterate over using 
 - the `json_each` function [in SQLite](https://www.sqlite.org/json1.html) and [Postgres](https://www.postgresql.org/docs/9.3/functions-json.html),
 - the [`JSON_TABLE`](https://dev.mysql.com/doc/refman/8.0/en/json-table-functions.html) function in MySQL (which you''ll need to wrap in a function, because SQLPage cannot parse the non-standard syntax of this function)
 - the [`OPENJSON`](https://learn.microsoft.com/fr-fr/sql/t-sql/functions/openjson-transact-sql?view=sql-server-ver16) function in Microsoft SQL Server.

The target page could then look like this:

```sql
insert into best_fruits(id) -- INSERT INTO ... SELECT ... runs the SELECT query and inserts the results into the table
select CAST(value AS integer) as id -- all values are transmitted by the browser as strings
from json_each($preferred_fruits); -- json_each returns a table with a "value" column for each element in the JSON array
```
', json('[{"component":"form"}, 
    {"name": "Fruit", "type": "select", "multiple": true, "description": "press ctrl to select multiple values", "options":
        "[{\"label\": \"Orange\", \"value\": 0}, {\"label\": \"Apple\", \"value\": 1}, {\"label\": \"Banana\", \"value\": 3}]"}
    ]')),
    ('form', 'This example illustrates the use of the `radio` type.
The `name` parameter is used to group the radio buttons together.
The `value` parameter is used to set the value that will be submitted when the user selects the radio button.
The `label` parameter is used to display a friendly name for the radio button.
The `description` parameter is used to display a helper text near the radio button.

We could also save all the options in a database table, and then run a simple query like

```sql
SELECT ''form'' AS component;
SELECT * FROM fruit_option;
```

In this example, depending on what the user clicks, the target `index.sql` page will be loaded with a the variable `$fruit` set to the string "1", "2", or "3".

    ', json('[{"component":"form", "method": "GET", "action": "index.sql"}, '||
    '{"name": "fruit", "type": "radio", "value": 1, "description": "An apple a day keeps the doctor away", "label": "Apple"}, '||
    '{"name": "fruit", "type": "radio", "value": 2, "description": "Oranges are a good source of vitamin C", "label": "Orange", "checked": true}, '||
    '{"name": "fruit", "type": "radio", "value": 3, "description": "Bananas are a good source of potassium", "label": "Banana"}'||
    ']')),
    ('form', 'When you want to include some information in the form data, but not display it to the user, you can use a hidden field.

This can be used to track simple data such as the current user''s id,
or to implement more complex flows, such as a multi-step form,
where the user is redirected to a different page after each step.

This can also be used to implement [CSRF protection](https://en.wikipedia.org/wiki/Cross-site_request_forgery#Synchronizer_token_pattern),
if your website has authenticated users that can perform sensitive actions through forms.
', json('[{"component":"form", "validate": "Delete", "validate_color": "red"}, 
    {"type": "hidden", "name": "user_id", "value": "place id here"},
    {"type": "hidden", "name": "csrf_token", "value": "place token here"},
    {"name": "confirm", "label": "Please type \"sensitive resource\" here to confirm the deletion", "required": true}
    ]')),
    ('form', 'This example illustrates the use of custom validation buttons and half-width fields.',
    json('[{"component":"form", "title": "User", "validate": "Create new user", "validate_color": "green", "reset": "Clear"},
    {"name": "first_name", "label": "First name", "placeholder": "John", "width": 4},
    {"name": "middle_name", "label": "Middle name", "placeholder": "Fitzgerald", "width": 4},
    {"name": "last_name", "label": "Last name", "placeholder": "Doe", "width": 4},
    {"name": "email", "label": "Email", "placeholder": "john.doe@gmail.com", "width": 12},
    {"name": "password", "label": "Password", "type": "password", "width": 6},
    {"name": "password_confirmation", "label": "Password confirmation", "type": "password", "width": 6},
    {"name": "terms", "label": "I accept the terms and conditions", "type": "checkbox", "required": true}
    ]')),
    ('form', '
## File upload

You can use the `file` type to allow the user to upload a file.
The file will be uploaded to the server, and you will be able to access it using the
[`sqlpage.uploaded_file_path`](functions.sql?function=uploaded_file_path#function) function.

Here is how you could save the uploaded file to a table in the database:

```sql
INSERT INTO uploaded_file(name, data) VALUES(:filename, sqlpage.uploaded_file_data_url(:filename))
```
',
    json('[{"component":"form", "title": "Upload a picture", "validate": "Upload", "action": "examples/handle_picture_upload.sql"}, 
    {"name": "my_file", "type": "file", "accept": "image/png, image/jpeg",  "label": "Picture", "description": "Upload a nice picture", "required": true}
    ]'))
;

INSERT INTO component(name, icon, description) VALUES
    ('chart', 'timeline', 'A component that plots data. Line, area, bar, and pie charts are all supported. Each item in the component is a data point in the graph.');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'chart', * FROM (VALUES
    -- top level
    ('title', 'The name of the chart.', 'TEXT', TRUE, TRUE),
    ('type', 'The type of chart: "line", "area", "bar", "column", "pie", "scatter", or "bubble".', 'TEXT', TRUE, FALSE),
    ('time', 'Whether the x-axis represents time. If set to true, the x values will be parsed and formatted as dates for the user.', 'BOOLEAN', TRUE, TRUE),
    ('ymin', 'The minimal value for the y-axis.', 'NUMBER', TRUE, TRUE),
    ('ymax', 'The maximum value for the y-axis.', 'NUMBER', TRUE, TRUE),
    ('xtitle', 'Title of the x axis, displayed below it.', 'TEXT', TRUE, TRUE),
    ('ytitle', 'Title of the y axis, displayed to its left.', 'TEXT', TRUE, TRUE),
    ('ztitle', 'Title of the z axis, displayed in tooltips.', 'TEXT', TRUE, TRUE),
    ('xticks', 'Number of ticks on the x axis.', 'NUMBER', TRUE, TRUE),
    ('marker', 'Marker size', 'NUMBER', TRUE, TRUE),
    ('labels', 'Whether to show the data labels on the chart or not.', 'BOOLEAN', TRUE, TRUE),
    ('color', 'The name of a color in which to display the chart. If there are multiple series in the chart, this parameter can be repeated multiple times.', 'COLOR', TRUE, TRUE),
    ('stacked', 'Whether to cumulate values from different series.', 'BOOLEAN', TRUE, TRUE),
    ('toolbar', 'Whether to display a toolbar at the top right of the chart, that offers downloading the data as CSV.', 'BOOLEAN', TRUE, TRUE),
    ('logarithmic', 'Display the y-axis in logarithmic scale.', 'BOOLEAN', TRUE, TRUE),
    ('horizontal', 'Displays a bar chart with horizontal bars instead of vertical ones.', 'BOOLEAN', TRUE, TRUE),
    -- item level
    ('x', 'The value of the point on the horizontal axis', 'NUMBER', FALSE, FALSE),
    ('y', 'The value of the point on the vertical axis', 'NUMBER', FALSE, FALSE),
    ('label', 'An alias for parameter "x"', 'NUMBER', FALSE, TRUE),
    ('value', 'An alias for parameter "y"', 'NUMBER', FALSE, TRUE),
    ('series', 'If multiple series are represented and share the same y-axis, this parameter can be used to distinguish between them.', 'TEXT', FALSE, TRUE)
) x;
INSERT INTO example(component, description, properties) VALUES
    ('chart', 'An area chart representing a time series, using the top-level property `time`.
    Ticks on the x axis are adjusted automatically, and ISO datetimes are parsed and displayed in a readable format.', json('[
    {
        "component": "chart",
        "title": "Quarterly Revenue",
        "type": "area",
        "color": "indigo",
        "marker": 5,
        "time": true
    },
        {"x":"2022-01-01T00:00:00Z","y":15},
        {"x":"2022-04-01T00:00:00Z","y":46},
        {"x":"2022-07-01T00:00:00Z","y":23},
        {"x":"2022-10-01T00:00:00Z","y":70},
        {"x":"2023-01-01T00:00:00Z","y":35},
        {"x":"2023-04-01T00:00:00Z","y":106},
        {"x":"2023-07-01T00:00:00Z","y":53}
    ]')),
    ('chart', 'A pie chart.', json('[{"component":"chart", "title": "Answers", "type": "pie", "labels": true}, '||
    '{"label": "Yes", "value": 65}, '||
    '{"label": "No", "value": 35}]')),
    ('chart', 'A basic bar chart', json('[{"component":"chart", "type": "bar", "title": "Quarterly Results", "horizontal": true}, '||
    '{"label": "Tom", "value": 35}, {"label": "Olive", "value": 15}]')),
    ('chart', 'A bar chart with multiple series.', json('[{"component":"chart", "title": "Expenses", "type": "bar", "stacked": true, "toolbar": true}, '||
    '{"series": "Marketing", "x": 2021, "value": 35}, '||
    '{"series": "Marketing", "x": 2022, "value": 15}, '||
    '{"series": "Human resources", "x": 2021, "value": 30}, '||
    '{"series": "Human resources", "x": 2022, "value": 55}]')),
    ('chart', 'A scatter plot with multiple custom options.',
    json('[
        {"component":"chart", "title": "Gross domestic product and its growth", "type": "scatter",
        "xtitle": "Growth Rate", "ytitle": "GDP (Trillions USD)", "marker": 8,
        "xmin": 0, "xmax": 10, "ymin": 0, "ymax": 25, "yticks": 10},

        {"series": "Brazil", "x": 2.5, "y": 2},
        {"series": "China", "x": 6.5, "y": 14},
        {"series": "United States", "x": 2.3, "y": 21},
        {"series": "France", "x": 1.5, "y": 3},
        {"series": "South Africa", "x": 0.9, "y": 0.3}
    ]'));

INSERT INTO component(name, icon, description) VALUES
    ('table', 'table', 'A table with optional filtering and sorting. Unlike most others, this component does not have a fixed set of item properties, any property that is used will be rendered directly as a column in the table.');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'table', * FROM (VALUES
    -- top level
    ('sort', 'Make the columns clickable to let the user sort by the value contained in the column.', 'BOOLEAN', TRUE, TRUE),
    ('search', 'Add a search bar at the top of the table, letting users easily filter table rows by value.', 'BOOLEAN', TRUE, TRUE),
    ('markdown', 'Set this to the name of a column whose content should be interpreted as markdown . Used to display rich text with links in the table. This argument can be repeated multiple times to intepret multiple columns as markdown.', 'TEXT', TRUE, TRUE),
    ('icon', 'Set this to the name of a column whose content should be interpreted as a tabler icon name. Used to display icons in the table. This argument can be repeated multiple times to intepret multiple columns as icons. Introduced in v0.8.0.', 'TEXT', TRUE, TRUE),
    ('align_right', 'Name of a column the contents of which should be right-aligned. This argument can be repeated multiple times to align multiple columns to the right. Introduced in v0.15.0.', 'TEXT', TRUE, TRUE),
    -- row level
    ('_sqlpage_css_class', 'For advanced users. Sets a css class on the table row. Added in v0.8.0.', 'TEXT', FALSE, TRUE),
    ('_sqlpage_color', 'Sets the background color of the row. Added in v0.8.0.', 'TEXT', FALSE, TRUE)
) x;

INSERT INTO example(component, description, properties) VALUES
    ('table', 'The most basic table.',
        json('[{"component":"table"}, {"a": 1, "b": 2}, {"a": 3, "b": 4}]')),
    ('table', 'A table of users with filtering and sorting.',
        json('[{"component":"table", "sort":true, "search":true}, '||
        '{"Forename": "Ophir", "Surname": "Lojkine", "Pseudonym": "lovasoa"},' ||
        '{"Forename": "Linus", "Surname": "Torvalds", "Pseudonym": "torvalds"}]')),
    ('table', 'A table that uses markdown to display links',
        json('[{"component":"table", "markdown": "Documentation", "icon": "icon", "sort": true, "search": true}, '||
        '{"icon": "table", "name": "Table", "description": "Displays SQL results as a searchable table.", "Documentation": "[docs](documentation.sql?component=table)", "_sqlpage_color": "red"},
        {"icon": "timeline", "name": "Chart", "description": "Show graphs based on numeric data.", "Documentation": "[docs](documentation.sql?component=chart)"}
        ]')),
    (
    'table',
    'A table with numbers',
    json(
        '[{"component":"table", "search": true, "sort": true, "align_right": ["Price ($)", "Amount in stock"]}, ' ||
         '{"id": 31456, "part_no": "MIC-ROCC-F-23-206-C", "Price ($)": 12, "Amount in stock": 5},
          {"id": 996, "part_no": "MIC-ROCC-F-24-206-A", "Price ($)": 1, "Amount in stock": 15},
          {"id": 131456, "part_no": "KIB-ROCC-F-13-205-B", "Price ($)": 127, "Amount in stock": 9}
        ]'
    )
    );


INSERT INTO component(name, icon, description) VALUES
    ('csv', 'download', 'A button that lets the user download data as a CSV file. Each column from the items in the component will map to a column in the resulting CSV.');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'csv', * FROM (VALUES
    -- top level
    ('separator', 'How individual values should be separated in the CSV. "," by default, set it to "\t" for tab-separated values.', 'TEXT', TRUE, TRUE),
    ('title', 'The text displayed on the download button.', 'TEXT', TRUE, FALSE),
    ('filename', 'The name of the file that should be downloaded (without the extension).', 'TEXT', TRUE, TRUE),
    ('icon', 'Name of the icon (from tabler-icons.io) to display in the button.', 'ICON', TRUE, TRUE),
    ('color', 'Color of the button', 'COLOR', TRUE, TRUE)
) x;

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
) x;

INSERT INTO example(component, description, properties) VALUES
    ('dynamic', 'Rendering a text paragraph dynamically.', json('[{"component":"dynamic", "properties": "[{\"component\":\"text\"}, {\"contents\":\"Blah\", \"bold\":true}]"}]'));

INSERT INTO component(name, icon, description) VALUES
    ('shell', 'layout-navbar', 'Personalize the "shell" surrounding your page contents. Used to set properties for the entire page.');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'shell', * FROM (VALUES
    -- top level
    ('title', 'The title of your page. Will be shown in a top bar above the page contents. Also usually displayed by web browsers as the name of the web page''s tab.', 'TEXT', TRUE, TRUE),
    ('description', 'A description of the page. It can be displayed by search engines when your page appears in their results.', 'TEXT', TRUE, TRUE),
    ('link', 'The target of the link in the top navigation bar.', 'URL', TRUE, TRUE),
    ('css', 'The URL of a CSS file to load and apply to the page.', 'URL', TRUE, TRUE),
    ('javascript', 'The URL of a Javascript file to load and execute on the page.', 'URL', TRUE, TRUE),
    ('image', 'The URL of an image to display next to the page title.', 'URL', TRUE, TRUE),
    ('icon', 'Name of an icon (from tabler-icons.io) to display next to the title in the navigation bar.', 'ICON', TRUE, TRUE),
    ('menu_item', 'Adds a menu item in the navigation bar at the top of the page. The menu item will have the specified name, and will link to as .sql file of the same name. A dropdown can be generated by passing a json object with a `title` and `submenu` properties.', 'TEXT', TRUE, TRUE),
    ('search_target', 'When this is set, a search field will appear in the top navigation bar, and load the specified sql file with an URL parameter named "search" when the user searches something.', 'TEXT', TRUE, TRUE),
    ('norobot', 'Forbids robots to save this page in their database and follow the links on this page. This will prevent this page to appear in Google search results for any query, for instance.', 'BOOLEAN', TRUE, TRUE),
    ('font', 'Name of a font to display the text in. This has to be a valid font name from fonts.google.com.', 'TEXT', TRUE, TRUE),
    ('font_size', 'Font size on the page, in pixels. Set to 18 by default.', 'INTEGER', TRUE, TRUE),
    ('language', 'The language of the page. This can be used by search engines and screen readers to determine in which language the page is written.', 'TEXT', TRUE, TRUE),
    ('refresh', 'Number of seconds after which the page should refresh. This can be useful to display dynamic content that updates automatically.', 'INTEGER', TRUE, TRUE),
    ('theme', 'Set to "dark" to use a dark theme.', 'TEXT', TRUE, TRUE),
    ('footer', 'Muted text to display in the footer of the page. This can be used to display a link to the terms and conditions of your application, for instance. By default, shows "Built with SQLPage". Supports links with markdown.', 'TEXT', TRUE, TRUE)
) x;

INSERT INTO example(component, description, properties) VALUES
    ('shell', '
This example contains the values used for the shell of the page you are currently viewing.

The `menu_item` property is used both in its simple string form, to generate a link named "functions" that points to "functions.sql",
and in its object form, to generate a dropdown menu named "Community" with links to the blog, the github repository, and the issues page.

The object form can be used directly only on database engines that have a native JSON type.
On other engines (such as SQLite), you can use the `dynamic` component to generate the same result.
',
     json('[{
            "component": "shell",
            "title": "SQLPage documentation",
            "link": "/",
            "menu_item": [
                {"link": "index.sql", "title": "Home"},
                {"title": "Community", "submenu": [
                    {"link": "blog.sql", "title": "Blog"},
                    {"link": "//github.com/lovasoa/sqlpage/issues", "title": "Issues"},
                    {"link": "//github.com/lovasoa/sqlpage/discussions", "title": "Discussions"},
                    {"link": "//github.com/lovasoa/sqlpage", "title": "Github"}
                ]},
                "functions",
                "components"
            ],
            "language": "en-US",
            "description": "Documentation for the SQLPage low-code web application framework.",
            "font": "Poppins",
            "icon": "book",
            "javascript": ["https://cdn.jsdelivr.net/npm/prismjs@1/components/prism-core.min.js", 
                           "https://cdn.jsdelivr.net/npm/prismjs@1/plugins/autoloader/prism-autoloader.min.js"],
            "css": "/prism-tabler-theme.css",
            "footer": "Official [SQLPage](https://sql.ophir.dev) documentation"
        }]'));
