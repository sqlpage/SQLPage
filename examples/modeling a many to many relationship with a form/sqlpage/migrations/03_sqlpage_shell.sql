-- Instead of adding a long select at the top of all pages.
-- We want to be able to to a simple query like:
-- SELECT * FROM shell LIMIT 1;

CREATE TABLE sqlpage_shell (
    component TEXT NOT NULL,
    title TEXT NOT NULL,
    link TEXT NOT NULL,
    menu_item TEXT NOT NULL,
    lang TEXT NOT NULL,
    description TEXT NOT NULL,
    font TEXT NOT NULL,
    font_size INTEGER NOT NULL,
    icon TEXT NOT NULL,
    footer TEXT NOT NULL
);

INSERT INTO sqlpage_shell (
component, title, link, menu_item, lang, description, font, font_size, icon, footer
) VALUES (
'shell', 'SQL Blog', '/', 'topics', 'en-US', 'A cool SQL-only blog', 'Playfair Display', 21, 'book', 'This blog is written entirely in SQL with [SQLPage](https://sql.datapage.app)'
);