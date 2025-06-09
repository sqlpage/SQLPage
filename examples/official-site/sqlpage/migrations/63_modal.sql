INSERT INTO component(name, icon, description, introduced_in_version) VALUES
    ('modal', 'app-window', 'Defines the content of a modal box. Useful for displaying additional information or help.', '0.36.0');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'modal', * FROM (VALUES
    ('title','Description of the modal box.','TEXT',TRUE,FALSE),
    ('close','The text to display in the Close button.','TEXT',TRUE,FALSE),
    ('contents','A paragraph of text to display, without any formatting, without having to make additional queries.','TEXT',FALSE,TRUE),
    ('contents_md','Rich text in the markdown format. Among others, this allows you to write bold text using **bold**, italics using *italics*, and links using [text](https://example.com).','TEXT',FALSE,TRUE),
    ('unsafe_contents_md','Markdown format with html blocks. Use this only with trusted content. See the html-blocks section of the Commonmark spec for additional info.','TEXT',FALSE,TRUE),
    ('scrollable','Create a scrollable modal that allows scroll the modal body.','BOOLEAN',TRUE,TRUE),
    ('class','Class attribute added to the container in HTML. It can be used to apply custom styling to this item through css.','TEXT',TRUE,TRUE),
    ('id','ID attribute added to the container in HTML. It can be used to target this item through css or for displaying this item.','TEXT',TRUE,FALSE)
) x;

INSERT INTO example(component, description, properties) VALUES
    ('modal', 
    'This example shows how to create a modal box that displays a paragraph of text. The modal window is opened with the help of a button.',
    json('[
        {"component": "modal","id": "my_modal","title": "A modal box","close": "Close"},
        {"contents":"I''m a modal window, and I allow you to display additional information or help for the user."},
        {"component": "button"},
        {"title":"Open","modal":"my_modal"}
        ]')
    );

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'button', * FROM (VALUES
    ('modal','Display the modal window corresponding to the specified ID.','TEXT',FALSE,TRUE)
) x;