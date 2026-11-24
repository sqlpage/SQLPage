---
icon: map
since: 0.8.0
category: display-data
summary: Plot locations and geographic shapes on an interactive map.
top:
  - name: attribution
    type: HTML
    description: Text to display at the bottom right of the map. Defaults to "© OpenStreetMap".
  - class
  - name: height
    type: INTEGER
    description: Height of the map, in pixels. Default to 350px
  - id
  - name: latitude
    type: REAL
    description: Latitude of the center of the map. If omitted, the map will be centered on its markers.
  - name: longitude
    type: REAL
    description: Longitude of the center of the map.
  - name: max_zoom
    type: INTEGER
    description: How far the map can be zoomed in. Defaults to 18. Added in v0.15.2.
  - name: tile_source
    type: URL
    description: Custom map tile images to use, as a URL. Defaults to "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png".
      Added in v0.15.2.
  - name: title
    type: TEXT
    description: A text header displayed above the map.
  - name: zoom
    type: REAL
    description: Zoom Level to apply to the map. Defaults to 5.
row:
  - name: color
    type: COLOR
    description: Background color of the marker on the map. Requires "icon" to be set.
  - name: description
    type: TEXT
    description: Plain text description of the marker, to be displayed in a tooltip when the marker is clicked.
  - name: description_md
    type: TEXT
    description: Description of the marker, in markdown, rendered in a tooltip when the marker is clicked.
  - name: geojson
    type: JSON
    description: A GeoJSON geometry (line, polygon, ...) to display on the map. Can be styled using geojson
      properties using the name of leaflet path options. Introduced in 0.15.1. Accepts raw strings in addition
      to JSON objects since 0.15.2.
  - name: icon
    type: ICON
    description: Name of the icon to use for the marker
  - name: latitude
    type: REAL
    required: true
    description: Latitude of the marker. Required only if geojson is not set.
  - name: link
    type: TEXT
    description: A link to associate to the marker's title. If set, the marker tooltip's title will be clickable
      and will open the link.
  - name: longitude
    type: REAL
    required: true
    description: Longitude of the marker. Required only if geojson is not set.
  - name: size
    type: INTEGER
    description: Size of the marker icon. Requires "icon" to be set. Introduced in 0.15.2.
  - name: title
    type: TEXT
    description: Title of the marker, displayed on hover and in the tooltip when the marker is clicked.
---

## Visualize SQL data on a map.

The map component displays a custom interactive map with markers on it.

In its simplest form, the component displays points on a map from a table of latitudes and longitudes.
But it can also be used by cartographers in combination with PostgreSQL's PostGIS or SQLite's spatialite,
to create custom visualizations of geospatial data.
Use the `geojson` property to generate rich maps from a GIS database.

### Example Use Cases

1. **Store Locator**: Build an interactive map to find the nearest store information using SQL-stored geospatial data.
2. **Delivery Route Optimization**: Visualize the results of delivery route optimization algorithms.
3. **Sales Heatmap**: Identify high-performing regions by mapping sales data stored in SQL.
4. **Real-Time Tracking**: Create dynamic dashboards that track vehicles, assets, or users live using PostGIS or MS SQL Server geospatial time series data. Use the [shell](?component=shell) component to auto-refresh the map.
5. **Demographic Insights**: Map customer demographics or trends geographically to uncover opportunities for growth or better decision-making.
