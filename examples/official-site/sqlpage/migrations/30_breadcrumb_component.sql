INSERT INTO component (name, description, icon, introduced_in_version)
VALUES (
        'breadcrumb',
        'A secondary navigation aid that helps users understand their location on a website or mobile application.',
        'dots',
        '0.18.0'
    );

INSERT INTO parameter (
        component,
        name,
        description,
        type,
        top_level,
        optional
    )
	VALUES (
        'breadcrumb',
        'title',
        'Hyperlink text to display.',
        'TEXT',
        FALSE,
        FALSE
    ),
    (
        'breadcrumb',
        'link',
        'Link to the page to display when the link is clicked. By default, the link refers to the current page, with a ''link'' parameter set to the link''s title.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'breadcrumb',
        'active',
        'Whether the link is active or not. Defaults to false.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'breadcrumb',
        'description',
        'Description of the link. This is displayed when the user hovers over the link.',
        'TEXT',
        FALSE,
        TRUE
    );

-- Insert example(s) for the component
INSERT INTO example(component, description, properties)
VALUES
    (
        'breadcrumb', 
        'Basic usage of the breadcrumb component', 
            JSON(
                '[
                {"component":"breadcrumb"},
                {"title":"Home","link":"/"},
                {"title":"Articles"},
                {"title":"Information Technology (IT)"}
                ]'
            )
    ),
    (
        'breadcrumb', 
        'Description of a link and selection of the current page.', 
            JSON(
                '[
                {"component":"breadcrumb"},
                {"title":"Home","link":"/"},
                {"title":"Articles","link":"blog.sql","description":"Stay informed with the latest news"},
                {"title":"Information Technology (IT)","link":"it.sql","active":true}
                ]'
            )
    );
