INSERT INTO component(name, icon, description, introduced_in_version) VALUES
    ('pagination', 'sailboat-2', '
Navigation links to go to the first, previous, next, or last page of a dataset. 
Useful when data is divided into pages, each containing a fixed number of rows.

This component only handles the display of pagination.
**Your sql queries are responsible for filtering data** based on the page number passed as a URL parameter.

This component is typically used in conjunction with a [table](?component=table),
[list](?component=list), or [card](?component=card) component.

The pagination component displays navigation buttons (first, previous, next, last) customizable with text or icons.

For large numbers of pages, an offset can limit the visible page links.

A minimal example of a SQL query that uses the pagination would be:
```sql
select ''table'' as component;
select * from my_table limit 100 offset $offset;

select ''pagination'' as component;
with recursive pages as (
    select 0 as offset
    union all
    select offset + 100 from pages
    where offset + 100 < (select count(*) from my_table)
)
select 
    (offset/100+1) as contents,
    sqlpage.link(sqlpage.path(), json_object(''offset'', offset)) as link,
    (offset/100+1 = $offset) as active from pages;
```

For more advanced usage, the [pagination guide](blog.sql?post=How+to+use+the+pagination+component) provides a complete tutorial.
', '0.40.0');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'pagination', * FROM (VALUES
    -- Top-level parameters
    ('first_link','A target URL to which the user should be directed to get to the first page. If none, the link is not displayed.','URL',TRUE,TRUE),
    ('previous_link','A target URL to which the user should be directed to get to the previous page. If none, the link is not displayed.','URL',TRUE,TRUE),
    ('next_link','A target URL to which the user should be directed to get to the next page. If none, the link is not displayed.','URL',TRUE,TRUE),
    ('last_link','A target URL to which the user should be directed to get to the last page. If none, the link is not displayed.','URL',TRUE,TRUE),
    ('first_title','The text displayed on the button to go to the first page.','TEXT',TRUE,TRUE),
    ('previous_title','The text displayed on the button to go to the previous page.','TEXT',TRUE,TRUE),
    ('next_title','The text displayed on the button to go to the next page.','TEXT',TRUE,TRUE),
    ('last_title','The text displayed on the button to go to the last page.','TEXT',TRUE,TRUE),
    ('first_disabled','disables the button to go to the first page.','BOOLEAN',TRUE,TRUE),
    ('previous_disabled','disables the button to go to the previous page.','BOOLEAN',TRUE,TRUE),
    ('next_disabled','Disables the button to go to the next page.','BOOLEAN',TRUE,TRUE),
    ('last_disabled','disables the button to go to the last page.','BOOLEAN',TRUE,TRUE),
    ('outline','Whether to use outline version of the pagination.','BOOLEAN',TRUE,TRUE),
    ('circle','Whether to use circle version of the pagination.','BOOLEAN',TRUE,TRUE),
    -- Item-level parameters (for each page)
    ('contents','Page number.','INTEGER',FALSE,FALSE),
    ('link','A target URL to which the user should be redirected to view the requested page of data.','URL',FALSE,TRUE),
    ('offset','Whether to use offset to show only a few pages at a time. Usefull if the count of pages is too large. Defaults to false','BOOLEAN',FALSE,TRUE),
    ('active','Whether the link is active or not. Defaults to false.','BOOLEAN',FALSE,TRUE)
) x;


-- Insert example(s) for the component
INSERT INTO example(component, description, properties)
VALUES (
        'pagination',
        'This is an extremely simple example of a pagination component that displays only the page numbers, with the first page being the current page.',
        JSON(
            '[
                {
                    "component": "pagination"
                },
                {
                    "contents": 1,
                    "link": "?component=pagination&page=1",
                    "active": true
                },
                {
                    "contents": 2,
                    "link": "?component=pagination&page=2"
                },
                {
                    "contents": 3,
                    "link": "?component=pagination&page=3"
                }
            ]'
        )
    ),
    (
        'pagination',
        'The ouline style adds a rectangular border to each navigation link.',
        JSON(
            '[
                {
                    "component": "pagination",
                    "outline": true
                },
                {
                    "contents": 1,
                    "link": "?component=pagination&page=1",
                    "active": true
                },
                {
                    "contents": 2,
                    "link": "?component=pagination&page=2"
                },
                {
                    "contents": 3,
                    "link": "?component=pagination&page=3"
                }
            ]'
        )
    ),
    (
        'pagination',
        'The circle style adds a circular border to each navigation link.',
        JSON(
            '[
                {
                    "component": "pagination",
                    "circle": true
                },
                {
                    "contents": 1,
                    "link": "?component=pagination&page=1",
                    "active": true
                },
                {
                    "contents": 2,
                    "link": "?component=pagination&page=2"
                },
                {
                    "contents": 3,
                    "link": "?component=pagination&page=3"
                }
            ]'
        )
    ),
    (
        'pagination',
        'The following example implements navigation links that can be enabled or disabled as needed. Since a navigation link does not appear if no link is assigned to it, you must always assign a link to display it as disabled.',
        JSON(
            '[
                {
                    "component": "pagination",
                    "first_link": "?component=pagination",
                    "first_disabled": true,
                    "previous_link": "?component=pagination",
                    "previous_disabled": true,
                    "next_link": "#?page=2",
                    "last_link": "#?page=3"

                },
                {
                    "contents": 1,
                    "link": "?component=pagination&page=1",
                    "active": true
                },
                {
                    "contents": 2,
                    "link": "?component=pagination&page=2"
                },
                {
                    "contents": 3,
                    "link": "?component=pagination&page=3"
                }
            ]'
        )
    ),
    (
        'pagination',
        'Instead of using icons, you can apply text to the navigation links.',
        JSON(
            '[
                {
                    "component": "pagination",
                    "first_title": "First",
                    "last_title": "Last",
                    "previous_title": "Previous",
                    "next_title": "Next",
                    "first_link": "?component=pagination",
                    "first_disabled": true,
                    "previous_link": "?component=pagination",
                    "previous_disabled": true,
                    "next_link": "#?page=2",
                    "last_link": "#?page=3"

                },
                {
                    "contents": 1,
                    "link": "?component=pagination&page=1",
                    "active": true
                },
                {
                    "contents": 2,
                    "link": "?component=pagination&page=2"
                },
                {
                    "contents": 3,
                    "link": "?component=pagination&page=3"
                }
            ]'
        )
    ),
    (
        'pagination',
        'If you have a large number of pages to display, you can use an offset to represent a group of pages.',
        JSON(
            '[
                {
                    "component": "pagination",
                    "first_link": "#?page=1",
                    "previous_link": "#?page=3",
                    "next_link": "#?page=4",
                    "last_link": "#?page=99"

                },
                {
                    "contents": 1,
                    "link": "?component=pagination&page=1"
                },
                {
                    "contents": 2,
                    "link": "?component=pagination&page=2"
                },
                {
                    "contents": 3,
                    "link": "?component=pagination&page=3"
                },
                {
                    "contents": 4,
                    "link": "?component=pagination&page=4",
                    "active": true
                },
                {
                    "contents": 5,
                    "link": "?component=pagination&page=5"
                },
                {
                    "contents": 6,
                    "link": "?component=pagination&page=6"
                },
                {
                    "offset": true
                },
                {
                    "contents": 99,
                    "link": "?component=pagination&page=99"
                },
            ]'
        )
    );
    