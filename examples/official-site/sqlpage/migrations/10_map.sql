INSERT INTO
    component (name, description, icon, introduced_in_version)
VALUES
    (
        'map',
        '

## Visualize SQL data on a map.

The map component displays a custom interactive map with markers on it.

In its simplest form, the component displays points on a map from a table of latitudes and longitudes.
But it can also be used by cartographers in combination with PostgreSQL''s PostGIS or SQLite''s spatialite,
to create custom visualizations of geospatial data.
Use the `geojson` property to generate rich maps from a GIS database.

### Example Use Cases

1. **Store Locator**: Build an interactive map to find the nearest store information using SQL-stored geospatial data.
2. **Delivery Route Optimization**: Visualize the results of delivery route optimization algorithms.
3. **Sales Heatmap**: Identify high-performing regions by mapping sales data stored in SQL.
4. **Real-Time Tracking**: Create dynamic dashboards that track vehicles, assets, or users live using PostGIS or MS SQL Server geospatial time series data. Use the [shell](?component=shell) component to auto-refresh the map.
5. **Demographic Insights**: Map customer demographics or trends geographically to uncover opportunities for growth or better decision-making.
',
        'map',
        '0.8.0'
    );

-- Insert the parameters for the http_header component into the parameter table
INSERT INTO
    parameter (
        component,
        name,
        description,
        type,
        top_level,
        optional
    )
VALUES
    (
        'map',
        'latitude',
        'Latitude of the center of the map. If omitted, the map will be centered on its markers.',
        'REAL',
        TRUE,
        TRUE
    ),
    (
        'map',
        'longitude',
        'Longitude of the center of the map.',
        'REAL',
        TRUE,
        TRUE
    ),
    (
        'map',
        'zoom',
        'Zoom Level to apply to the map. Defaults to 5.',
        'REAL',
        TRUE,
        TRUE
    ),
    (
        'map',
        'max_zoom',
        'How far the map can be zoomed in. Defaults to 18. Added in v0.15.2.',
        'INTEGER',
        TRUE,
        TRUE
    ),
    (
        'map',
        'tile_source',
        'Custom map tile images to use, as a URL. Defaults to "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png". Added in v0.15.2.',
        'URL',
        TRUE,
        TRUE
    ),
    (
        'map',
        'attribution',
        'Text to display at the bottom right of the map. Defaults to "© OpenStreetMap".',
        'HTML',
        TRUE,
        TRUE
    ),
    (
        'map',
        'latitude',
        'Latitude of the marker. Required only if geojson is not set.',
        'REAL',
        FALSE,
        FALSE
    ),
    (
        'map',
        'longitude',
        'Longitude of the marker. Required only if geojson is not set.',
        'REAL',
        FALSE,
        FALSE
    ),
    (
        'map',
        'title',
        'Title of the marker, displayed on hover and in the tooltip when the marker is clicked.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'map',
        'link',
        'A link to associate to the marker''s title. If set, the marker tooltip''s title will be clickable and will open the link.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'map',
        'description',
        'Plain text description of the marker, to be displayed in a tooltip when the marker is clicked.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'map',
        'description_md',
        'Description of the marker, in markdown, rendered in a tooltip when the marker is clicked.',
        'TEXT',
        FALSE,
        TRUE
    ),
    (
        'map',
        'icon',
        'Name of the icon to use for the marker',
        'ICON',
        FALSE,
        TRUE
    ),
    (
        'map',
        'color',
        'Background color of the marker on the map. Requires "icon" to be set.',
        'COLOR',
        FALSE,
        TRUE
    ),
    (
        'map',
        'geojson',
        'A GeoJSON geometry (line, polygon, ...) to display on the map. Can be styled using geojson properties using the name of leaflet path options. Introduced in 0.15.1. Accepts raw strings in addition to JSON objects since 0.15.2.',
        'JSON',
        FALSE,
        TRUE
    ),
    (
        'map',
        'size',
        'Size of the marker icon. Requires "icon" to be set. Introduced in 0.15.2.',
        'INTEGER',
        FALSE,
        TRUE
    );

-- Insert an example usage of the map component into the example table
INSERT INTO
    example (component, description, properties)
VALUES
    (
        'map',
        '
### Adding a marker to a map

Showing how to place a marker on a map. Useful for basic location displays like showing a single office location, event venue, or point of interest. The marker shows basic hover and click interactions.
',
        JSON (
            '[{ "component": "map" }, { "title": "New Delhi", "latitude": 28.6139, "longitude": 77.2090 }]'
        )
    ),
    (
        'map',
        '
### Advanced map customization using GeoJSON and custom map tiles

This example demonstrates using topographic map tiles, custom marker styling,
and clickable markers that link to external content - perfect for educational or tourism applications.

It uses [GeoJSON](https://en.wikipedia.org/wiki/GeoJSON) to display polygons and lines.

 - You can generate GeoJSON data from PostGIS geometries using the [`ST_AsGeoJSON`](https://postgis.net/docs/ST_AsGeoJSON.html) function.
 - In spatialite, you can use the [`AsGeoJSON`](https://www.gaia-gis.it/gaia-sins/spatialite-sql-5.1.0.html#p3misc) function.
 - In MySQL, you can use the [`ST_AsGeoJSON()`](https://dev.mysql.com/doc/refman/8.0/en/spatial-geojson-functions.html#function_st-asgeojson) function.
',
        JSON (
            '[{ "component": "map", "zoom": 5, "max_zoom": 8, "height": 600, "latitude": -25, "longitude": 28, "tile_source": "https://{s}.tile.opentopomap.org/{z}/{x}/{y}.png", "attribution": "" },
            { "icon": "peace",
            "size": 20,
            "link": "https://en.wikipedia.org/wiki/Nelson_Mandela",
            "geojson": "{\"type\":\"Feature\", \"properties\": { \"title\":\"Mvezo, Birth Place of Nelson Mandela\" }, \"geometry\": { \"type\":\"Point\", \"coordinates\": [28.49, -31.96] }}"}]'
        )
    ),
    (
        'map',
        '
### Maps with links and rich descriptions

Demonstrates how to create an engaging map with custom icons, colors, rich descriptions with markdown support, and connecting points with lines.
Perfect for visualizing multi-dimensional relationships between points on a map, like routes between locations.
',
        JSON (
            '[
            { "component": "map", "title": "Paris", "zoom": 11, "latitude": 48.85, "longitude": 2.34 },
            { "title": "Notre Dame", "icon": "building-castle", "color": "indigo", "latitude": 48.8530, "longitude": 2.3498, "description_md": "A beautiful cathedral.", "link": "https://en.wikipedia.org/wiki/Notre-Dame_de_Paris" },
            { "title": "Eiffel Tower", "icon": "tower", "color": "red", "latitude": 48.8584, "longitude": 2.2945, "description_md": "A tall tower. [Wikipedia](https://en.wikipedia.org/wiki/Eiffel_Tower)" },
            { "title": "Tower to Cathedral", "geojson": {"type": "LineString", "coordinates": [[2.2945, 48.8584], [2.3498, 48.8530]]}, "color": "teal", "description": "A nice 45 minutes walk." }
            ]'
        )
    ),
    (
        'map',
        '
### Abstract geometric visualizations

Example showing how to create abstract geometric visualizations without a base map.
Useful for displaying spatial data that doesn''t need geographic context, like floor plans, seating charts,
or abstract 2D data visualizations.
',
        JSON (
            '[
            { "component": "map", "tile_source": false },
            { "title": "MySQL", 
                "color": "red", "description": "The litteral red square",
                "geojson": {"type": "Polygon", "coordinates": [[[0, 0], [0, 4], [4, 4], [4, 0], [0, 0]]]}
            },
            {
                "title": "SQLite",
                "color": "blue", "description": "This 2D shape was generated by a SQL query.",
                "geojson": {"type": "Polygon", "coordinates": [[[5, 0], [9, 0], [7, 4], [5, 0]]]}
            }
            ]'
        )
    );