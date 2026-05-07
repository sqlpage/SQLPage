INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'card', * FROM (VALUES
    -- item level
    ('top_image_lasy', 'Whether the top image must be lazily or eagerly loaded. Defaults to false, meaning eagerly', 'BOOLEAN', FALSE, TRUE),
    ('top_image_width', 'Specify the top image width, in pixel. Help preventing a layout shift', 'INTEGER', FALSE, TRUE),
    ('top_image_height', 'Specify the top image height, in pixel. Help preventing a layout shift', 'INTEGER', FALSE, TRUE)
) x;

