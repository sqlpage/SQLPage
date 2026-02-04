INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'form', * FROM (VALUES
    -- top level
    ('validate_icon', 'Name of an icon to be displayed on the left side of the submit button.', 'ICON', TRUE, TRUE),
    ('reset_icon', 'Name of an icon to be displayed on the left side of the reset button.', 'ICON', TRUE, TRUE),
    ('reset_color', 'The color of the button at the bottom of the form that resets the form to its original state. Omit this property to use the default color.', 'COLOR', TRUE, TRUE)
    )

