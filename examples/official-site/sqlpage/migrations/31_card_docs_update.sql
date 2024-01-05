DELETE FROM component WHERE name = 'card';

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
    ('active', 'Whether this item in the grid is considered "active". Active items are displayed more prominently.', 'BOOLEAN', FALSE, TRUE),
    ('embed', 'A url whose contents will be fetched and injected into the body of this card.
        This can be used to inject arbitrary html content, but is especially useful for injecting
        the output of other sql files rendered by SQLPage. For the latter case you can pass the
        `?_sqlpage_embed` query parameter, which will skip the shell layout', 'TEXT', FALSE, TRUE)
) x;

