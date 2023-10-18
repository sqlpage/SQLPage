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
    ('link', 'The URL to which the button should navigate when clicked.', 'URL', FALSE, TRUE),
    ('color', 'The color of the button (e.g., red, green, blue, but also primary, warning, danger, orange, etc.).', 'TEXT', FALSE, TRUE),
    ('title', 'The text displayed on the button.', 'TEXT', FALSE, TRUE),
    ('disabled', 'Whether the button is disabled or not.', 'BOOLEAN', FALSE, TRUE),
    ('outline', 'Outline color of the button (e.g. red, purple, ...)', 'TEXT', FALSE, TRUE),
    ('space_after', 'Whether there should be extra space to the right of the button. In a line of buttons, this will put the buttons before this one on the left, and the ones after on the right.', 'BOOLEAN', FALSE, TRUE),
    ('icon', 'An icon (from tabler-icons) to be displayed on the left side of the button.', 'TEXT', FALSE, TRUE)
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
        {"link":"#", "color":"green", "title":"Save" },
        {"link":"#", "color":"orange", "title":"Cancel", "space_after":true},
        {"link":"#", "outline":"indigo", "title":"Preview" }]')
    );