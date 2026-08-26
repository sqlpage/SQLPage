INSERT INTO component(name, icon, description, introduced_in_version) VALUES
    ('facet', 'filter', '
Navigation links for filtering a dataset by a category, status, owner, or other attribute.

This component only renders the filter navigation. **Your SQL query is responsible for filtering the data** based on the URL parameter selected by the user.

Use it alongside a [table](?component=table), [list](?component=list), or [card](?component=card). The links use GET parameters, so the selected filter can be bookmarked and shared.

Set `compact` to display the facets in a dropdown, which is useful for a moderate number of choices or on narrow screens. Use `dropdown_title` to show the active facet in the closed dropdown. `all_link` and `all_title` add a link that clears the filter.

The following portable pattern filters a table by category. `sqlpage.link` preserves the current page path while safely generating the URL.

```sql
select ''table'' as component;
select title, category
from my_table
where $category is null or category = $category;

select ''facet'' as component,
    ''Category'' as description,
    sqlpage.link(sqlpage.path(), json_object(''category'', null)) as all_link,
    $category is null as all_active;
select distinct category as title,
    sqlpage.link(sqlpage.path(), json_object(''category'', category)) as link,
    category = $category as active
from my_table
order by category;
```
', '0.46.0');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'facet', * FROM (VALUES
    -- Top-level parameters
    ('description','The facet category label. In compact mode, it is shown on the dropdown button when `dropdown_title` is omitted. If omitted, the button displays "Choose facet".','TEXT',TRUE,TRUE),
    ('dropdown_title','Text shown on the compact-mode dropdown button. Use it to show the active facet. Defaults to `description`.','TEXT',TRUE,TRUE),
    ('compact','Displays the facets in a dropdown instead of links.','BOOLEAN',TRUE,TRUE),
    ('all_link','URL that clears the filter. If omitted, no All link is displayed.','URL',TRUE,TRUE),
    ('all_title','Text for the link that clears the filter. Defaults to "ALL".','TEXT',TRUE,TRUE),
    ('all_active','Whether the link that clears the filter is active. Defaults to false.','BOOLEAN',TRUE,TRUE),
    -- Item-level parameters (for each facet)
    ('title','Facet title.','TEXT',FALSE,FALSE),
    ('link','URL for the facet.','URL',FALSE,FALSE),
    ('active','Whether the link is active or not. Defaults to false.','BOOLEAN',FALSE,TRUE)
) x;

-- Insert example(s) for the component
INSERT INTO example(component, description, properties)
VALUES (
        'facet',
        'A category selector with an All link. In an application, the SQL pattern above uses the chosen URL parameter to filter the displayed data.',
        JSON(
            '[
                {
                    "component": "table"
                },
                {
                    "name": "USS Enterprise (NCC-1701)",
                    "class": "Constitution"
                },
                {
                    "name": "USS Exeter (NCC-1672)",
                    "class": "Galaxy"
                },
                {
                    "name": "USS Exeter (NCC-1672)",
                    "class": "Constitution"
                },
                {
                    "component": "facet",
                    "description": "Classes",
                    "all_link": "?component=facet",
                    "all_title": "All",
                    "all_active": false
                },
                {
                    "title": "Constitution",
                    "link": "?component=facet&class=Constitution",
                    "active": true
                },
                {
                    "title": "Galaxy",
                    "link": "?component=facet&class=Galaxy"
                }
            ]'
        )
    ),
    (
        'facet',
        'Compact mode displays the facet choices in a dropdown.',
        JSON(
            '[
                {
                    "component": "table"
                },
                {
                    "name": "USS Enterprise (NCC-1701)",
                    "class": "Constitution"
                },
                {
                    "name": "USS Exeter (NCC-1672)",
                    "class": "Galaxy"
                },
                {
                    "name": "USS Exeter (NCC-1672)",
                    "class": "Constitution"
                },
                {
                    "component": "facet",
                    "description": "Classes",
                    "all_link": "?component=facet",
                    "all_title": "All",
                    "all_active": false,
                    "compact": true,
                    "dropdown_title": "Constitution"
                },
                {
                    "title": "Constitution",
                    "link": "?component=facet&class=Constitution",
                    "active": true
                },
                {
                    "title": "Galaxy",
                    "link": "?component=facet&class=Galaxy"
                }
            ]'
        )
    );
