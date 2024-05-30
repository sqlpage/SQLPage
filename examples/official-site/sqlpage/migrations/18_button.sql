-- Button Component Documentation

-- Component Definition
INSERT INTO component(name, icon, description, introduced_in_version) VALUES
    ('button', 'hand-click', 'A versatile button component do display one or multiple button links of different styles.', '0.14.0');

-- Inserting parameter information for the button component
INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'button', * FROM (VALUES
    -- Top-level parameters (for the whole button list)
    ('justify', 'The horizontal alignment of the button list (e.g., start, end, center, between).', 'TEXT', TRUE, TRUE),
    ('size', 'The size of the buttons (e.g., sm, lg).', 'TEXT', TRUE, TRUE),
    ('shape', 'Shape of the buttons (e.g., pill, square)', 'TEXT', TRUE, TRUE),
    -- Item-level parameters (for each button)
    ('link', 'The URL to which the button should navigate when clicked. If the form attribute is specified, then this overrides the page to which the form is submitted.', 'URL', FALSE, TRUE),
    ('color', 'The color of the button (e.g., red, green, blue, but also primary, warning, danger, orange, etc.).', 'COLOR', FALSE, TRUE),
    ('title', 'The text displayed on the button.', 'TEXT', FALSE, TRUE),
    ('disabled', 'Whether the button is disabled or not.', 'BOOLEAN', FALSE, TRUE),
    ('outline', 'Outline color of the button (e.g. red, purple, ...)', 'COLOR', FALSE, TRUE),
    ('space_after', 'Whether there should be extra space to the right of the button. In a line of buttons, this will put the buttons before this one on the left, and the ones after on the right.', 'BOOLEAN', FALSE, TRUE),
    ('icon_after', 'Name of an icon to display after the text in the button', 'ICON', FALSE, TRUE),
    ('icon', 'Name of an icon to be displayed on the left side of the button.', 'ICON', FALSE, TRUE),
    ('form', 'Identifier (id) of the form to which the button should submit.', 'TEXT', FALSE, TRUE),
    ('id', 'HTML Identifier to add to the button element.', 'TEXT', FALSE, TRUE)
) x;

-- Inserting example information for the button component
INSERT INTO example(component, description, properties) VALUES
    ('button', 'A basic button with a link', 
    json('[{"component":"button"}, {"link":"/documentation.sql", "title":"Enabled"}, {"link":"#", "title":"Disabled", "disabled":true}]'))
;


INSERT INTO example(component, description, properties) VALUES
    ('button', 'A button with a custom shape, size, and outline color',
    json('[{"component":"button", "size":"sm", "shape":"pill" }, 
        {"title":"Purple", "outline":"purple" }, 
        {"title":"Orange", "outline":"orange" }, 
        {"title":"Red", "outline":"red" }]')
    );

INSERT INTO example(component, description, properties) VALUES
    ('button', 'A list of buttons aligned in the center',
    json('[{"component":"button", "justify":"center"}, 
        {"link":"#", "color":"light", "title":"Light"}, 
        {"link":"#", "color":"success", "title":"Success"},
        {"link":"#", "color":"info", "title":"Info"},
        {"link":"#", "color":"dark", "title":"Dark"},
        {"link":"#", "color":"warning", "title":"Warning"}]')
    );

INSERT INTO example(component, description, properties) VALUES
    ('button', 'Buttons with icons and different sizes',
    json('[{"component":"button", "size":"lg" }, 
        {"link":"#", "outline":"azure", "title":"Edit", "icon":"edit"},
        {"link":"#", "outline":"danger", "title":"Delete", "icon":"trash"}]')
    );

INSERT INTO example(component, description, properties) VALUES
    ('button', 'A row of square buttons with spacing in between',
    json('[{"component":"button", "shape":"square"}, 
        {"link":"#", "color":"green", "title":"Save", "icon": "device-floppy" },
        {"link":"#", "title":"Cancel", "space_after":true, "tooltip": "This will delete your draft"},
        {"link":"#", "outline":"indigo", "title":"Preview", "icon_after": "corner-down-right", "tooltip": "View temporary draft" }]')
    );

INSERT INTO example(component, description, properties) VALUES
    ('button', 'Multiple buttons sending the same form to different pages.

We use `'''' AS validate` to remove the submit button from inside the form itself,
and instead use the button component to submit the form to pages with different GET variables.

In the target page, we could then use the GET variable `$action` to determine what to do with the form data.
    ',
    json('[{"component":"form", "id": "poem", "validate": ""},
        {"type": "textarea", "name": "Poem", "placeholder": "Write a poem"},
        {"component":"button"}, 
        {"link":"?action=save", "form":"poem", "color":"primary", "title":"Save" },
        {"link":"?action=preview", "form":"poem", "outline":"yellow", "title":"Preview" }]')
    );