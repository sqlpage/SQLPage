INSERT INTO component(name, icon, description, introduced_in_version) VALUES
    ('partition', 'stack-front', '
Provides a mechanism to define custom pagination rules. 
It allows splitting records on the generated page based on a user-specified partitioning criterion, 
ensuring controlled data distribution across pages.

The `partition` component is typically used alongside the [table](?component=table) component to define which rows are displayed.
* The component uses the GET method to send the active partition selection parameters to the server.
* `compact` overrides the default link-based display when enabled.
* `all_link` and `all_title` work together to provide a fallback for viewing all data in a single view.
* `id` and `class` are standard HTML attributes for styling and DOM manipulation.

The SQL query below uses the [Chinook](https://www.sqlitetutorial.net/sqlite-sample-database) SQLite database. In this example, we use the partition component 
to display the music albums of an artist. The `$id` parameter passed in the URL corresponds to the primary key 
of the selected artist.

```sql
select ''table'' as component;
select title 
from 
    albums alb,
    artists art
where 
    alb.ArtistId = art.artistid
and (($id IS NULL) or ($id IS NOT NULL and art.artistid = $id))

select 
    ''partition'' as component,
    ''Artists'' as description,
    ''?component=partition'' as all_link,
    ($id IS NULL) as all_active;
select distinct
    art.Name as title,
    concat(''?component=partition&id='',art.artistid) as link,
    (art.artistid = $id) as active
from
    artists art,
    albums alb
where 
    alb.ArtistId = art.artistid;
```
', '0.46.0');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'partition', * FROM (VALUES
    -- Top-level parameters
    ('description','Specifies the type of partitions. In compact mode, if none, the placeholder text "Choose partition" is displayed.','TEXT',TRUE,TRUE),
    ('compact','Allows selecting the partition display mode. If the compact attribute is set to TRUE, partitions are displayed in a dropdown list. By default, partitions are shown as links.','BOOLEAN',TRUE,TRUE),
    ('all_link','Add a link to display all data. If none, the link is not displayed.','URL',TRUE,TRUE),
    ('all_title','Text used for the link to display all data. If none, the placeholder "ALL" is displayed.','TEXT',TRUE,TRUE),
    ('all_active','Whether the link to display all data is active or not. Defaults to false.','TEXT',TRUE,TRUE),
    -- Item-level parameters (for each page)
    ('title','Partition title.','TEXT',FALSE,FALSE),
    ('link','A target URL to which the user should be redirected to view the requested partition.','URL',FALSE,FALSE),
    ('active','Whether the link is active or not. Defaults to false.','BOOLEAN',FALSE,TRUE)
) x;


-- Insert example(s) for the component
INSERT INTO example(component, description, properties)
VALUES (
        'partition',
        'Classification of starships in Star Trek. Class selection is performed using links.',
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
                    "component": "partition",
                    "description": "Classes",
                    "all_link": "?component=partition",
                    "all_title": "All",
                    "all_active": true
                },
                {
                    "title": "Constitution",
                    "link": "?component=partition&class=1",
                },
                {
                    "title": "Galaxy",
                    "link": "?component=partition&class=2"
                }               
            ]'
        )
    ),
    (
        'partition',
        'The second example uses the compact display mode for partition names. This mode is suitable when many partitions are available and for display on mobile devices.',
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
                    "component": "partition",
                    "description": "Classes",
                    "all_link": "?component=partition",
                    "all_title": "All",
                    "all_active": true,
                    "compact": true
                },
                {
                    "title": "Constitution",
                    "link": "?component=partition&class=1",
                },
                {
                    "title": "Galaxy",
                    "link": "?component=partition&class=2"
                }               
            ]'
        )
    );

