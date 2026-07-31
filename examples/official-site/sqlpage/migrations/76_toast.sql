INSERT INTO component(name, icon, description, introduced_in_version) VALUES
    ('toast', 'notification', '
Displays a brief notification above the page. Each top-level `toast` row creates one notification, and consecutive toasts at the same position are queued in a shared stack.

Ordinary notifications use `role="status"` and `aria-live="polite"`. Colored variants retain a readable contrasting foreground. Automatic dismissal and the optional manual close control are configured independently.', '0.46.0');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'toast', * FROM (VALUES
    ('title', 'Optional notification heading.', 'TEXT', TRUE, TRUE),
    ('description', 'Escaped plain-text body. This is used only when `description_md` is not supplied.', 'TEXT', TRUE, TRUE),
    ('description_md', 'Rich-text alternative to `description`, rendered as Markdown. When both properties are supplied, `description_md` takes precedence.', 'TEXT', TRUE, TRUE),
    ('icon', 'Optional [Tabler icon](https://tabler.io/icons) name.', 'ICON', TRUE, TRUE),
    ('color', 'Optional Tabler color. The default is the neutral toast appearance; colored variants use the matching contrasting foreground utility.', 'COLOR', TRUE, TRUE),
    ('dismissible', 'Whether to render an accessible manual close button. Defaults to false and is independent of automatic dismissal.', 'BOOLEAN', TRUE, TRUE),
    ('duration', 'Automatic dismissal delay in milliseconds. Defaults to 5000. Set to 0 to keep the toast visible until manually dismissed (when `dismissible` is true) or the page is left.', 'INTEGER', TRUE, TRUE),
    ('position', 'Screen placement: `top-start`, `top-center`, `top-end`, `bottom-start`, `bottom-center`, or `bottom-end`. Defaults to `top-end`; invalid values safely fall back to that default.', 'TEXT', TRUE, TRUE),
    ('trigger', 'Optional URL fragment that opens the toast without reloading the page, with or without the leading `#`. When set, the toast does not open on page load and can be opened repeatedly by a link or button whose target is that fragment. Multiple toasts can share a trigger to open as a stack.', 'TEXT', TRUE, TRUE),
    ('id', 'Optional stable HTML ID for the toast.', 'TEXT', TRUE, TRUE),
    ('class', 'Optional custom CSS class appended to the toast.', 'TEXT', TRUE, TRUE)
) x;

INSERT INTO example(component, description, properties) VALUES
    ('toast', 'A “Changes saved” confirmation that disappears automatically after the default 5000 milliseconds.', json('[
        {"component":"toast","id":"toast-auto","title":"Changes saved","description":"Your changes have been saved.","icon":"check","color":"green"}
    ]')),
    ('toast', 'A persistent error notification that disables automatic dismissal and requires the user to activate its close button.', json('[
        {"component":"toast","id":"toast-dismissible","trigger":"persistent-error","title":"Could not save","description":"Review the highlighted fields and try again.","icon":"alert-triangle","color":"red","duration":0,"dismissible":true},
        {"component":"toast","id":"toast-nondismissible","trigger":"persistent-status","title":"Connection unavailable","description":"This persistent notification has no manual close control.","duration":0,"dismissible":false},
        {"component":"button"},
        {"title":"Show dismissible error","link":"#persistent-error","color":"red"},
        {"title":"Show non-dismissible status","link":"#persistent-status"}
    ]')),
    ('toast', 'A persistent rich Markdown notification. Markdown takes precedence over plain text and renders emphasis and a link as HTML, while plain-text content remains escaped.', json('[
        {"component":"toast","id":"toast-markdown","trigger":"rich-notifications","title":"Release available","description":"<strong>This fallback stays escaped</strong>","description_md":"Version **2.0** is ready. [Read the notes](https://example.com/releases).","duration":0},
        {"component":"toast","id":"toast-plain","trigger":"rich-notifications","description":"<strong>Plain text stays escaped</strong>","duration":0},
        {"component":"button"},
        {"title":"Show rich notifications","link":"#rich-notifications"}
    ]')),
    ('toast', 'Several persistent queued notifications. Toasts with the same position share a container and stack instead of overlapping.', json('[
        {"component":"toast","id":"toast-stack-one","trigger":"queued-notifications","title":"Import started","description":"Preparing records.","duration":0},
        {"component":"toast","id":"toast-stack-two","trigger":"queued-notifications","title":"Import running","description":"Processing records.","duration":0},
        {"component":"toast","id":"toast-short","trigger":"queued-notifications","title":"Temporary update","description":"This message closes shortly.","duration":2000},
        {"component":"button"},
        {"title":"Show queued notifications","link":"#queued-notifications"}
    ]')),
    ('toast', 'A notification placed at the bottom center of the screen instead of the default top end.', json('[
        {"component":"toast","id":"toast-bottom-center","trigger":"bottom-notification","title":"Download ready","description":"Your export is ready.","position":"bottom-center","duration":0},
        {"component":"button"},
        {"title":"Show bottom notification","link":"#bottom-notification"}
    ]'));
