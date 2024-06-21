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
    ('color', 'The color of the button (e.g., red, green, blue, but also primary, warning, danger, etc.).', 'COLOR', FALSE, TRUE),
    ('title', 'The text displayed on the button.', 'TEXT', FALSE, TRUE),
    ('tooltip', 'Text displayed when the user hovers over the button.', 'TEXT', FALSE, TRUE),
    ('disabled', 'Whether the button is disabled or not.', 'BOOLEAN', FALSE, TRUE),
    ('outline', 'Outline color of the button (e.g. red, purple, ...)', 'COLOR', FALSE, TRUE),
    ('space_after', 'Whether there should be extra space to the right of the button. In a line of buttons, this will put the buttons before this one on the left, and the ones after on the right.', 'BOOLEAN', FALSE, TRUE),
    ('icon_after', 'Name of an icon to display after the text in the button', 'ICON', FALSE, TRUE),
    ('icon', 'Name of an icon to be displayed on the left side of the button.', 'ICON', FALSE, TRUE),
    ('img', 'Path to image file (relative. relative to web root or URL) to be displayed on the button.', 'TEXT', FALSE, TRUE),
    ('narrow', 'Whether to trim horizontal padding.', 'BOOLEAN', FALSE, TRUE),
    ('form', 'Identifier (id) of the form to which the button should submit.', 'TEXT', FALSE, TRUE),
    ('rel', '"nofollow" when the contents of the target link are not endorsed, "noopener" when the target is not trusted, and "noreferrer" to hide where the user came from when they open the link.', 'TEXT', FALSE, TRUE),
    ('target', '"_blank" to open the link in a new tab, "_self" to open it in the same tab, "_parent" to open it in the parent frame, or "_top" to open it in the full body of the window.', 'TEXT', FALSE, TRUE),
    ('download', 'If defined, the link will download the target instead of navigating to it. Set the value to the desired name of the downloaded file.', 'TEXT', FALSE, TRUE),
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
        {"link":"#", "color":"warning", "title":"Warning"},
        {"link":"#", "color":"danger", "title":"Narrow"}]')
    );

INSERT INTO example(component, description, properties) VALUES
    ('button', 'Icon buttons using the narrow property',
    json('[{"component":"button"}, 
        {"link":"#", "narrow":true, "icon":"edit", "color":"primary", "tooltip":"Edit" },
        {"link":"#", "narrow":true, "icon":"trash", "color":"danger", "tooltip":"Delete" },
        {"link":"#", "narrow":true, "icon":"corner-down-right", "color":"info", "tooltip":"Preview" },
        {"link":"#", "narrow":true, "icon":"download", "color":"success", "tooltip":"Download" },
        {"link":"#", "narrow":true, "icon":"upload", "color":"warning", "tooltip":"Upload" },
        {"link":"#", "narrow":true, "icon":"info-circle", "color":"cyan", "tooltip":"Info" },
        {"link":"#", "narrow":true, "icon":"help-circle", "color":"purple", "tooltip":"Help" },
        {"link":"#", "narrow":true, "icon":"settings", "color":"indigo", "tooltip":"Settings" }]')
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

INSERT INTO example(component, description, properties) VALUES
    ('button', 'A button that downloads a file when clicked, and prevents search engines from following the link.',
    json('[{"component":"button"}, 
        {"link":"/sqlpage_introduction_video.webm",
            "title":"Download Video",
            "icon":"download",
            "download":"Introduction Video.webm",
            "rel":"nofollow"
        }]')
    );

INSERT INTO example(component, description, properties) VALUES
    ('button', 'A button with an image-based icon.',
    json('[{"component":"button"}, 
        {"link":"https://en.wikipedia.org/wiki/File:Globe.svg",
            "title":"Open an article",
            "img":"https://upload.wikimedia.org/wikipedia/commons/f/fa/Globe.svg"
        }]')
    );
