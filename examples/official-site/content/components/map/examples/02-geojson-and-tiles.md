---
rows:
  - component: map
    zoom: 5
    max_zoom: 8
    height: 600
    latitude: -25
    longitude: 28
    tile_source: https://{s}.tile.opentopomap.org/{z}/{x}/{y}.png
    attribution: ''
  - icon: peace
    size: 20
    link: https://en.wikipedia.org/wiki/Nelson_Mandela
    geojson: '{"type":"Feature", "properties": { "title":"Mvezo, Birth Place of Nelson Mandela" }, "geometry":
      { "type":"Point", "coordinates": [28.49, -31.96] }}'
---

### Advanced map customization using GeoJSON and custom map tiles

This example demonstrates using topographic map tiles, custom marker styling,
and clickable markers that link to external content - perfect for educational or tourism applications.

It uses [GeoJSON](https://en.wikipedia.org/wiki/GeoJSON) to display polygons and lines.

 - You can generate GeoJSON data from PostGIS geometries using the [`ST_AsGeoJSON`](https://postgis.net/docs/ST_AsGeoJSON.html) function.
 - In spatialite, you can use the [`AsGeoJSON`](https://www.gaia-gis.it/gaia-sins/spatialite-sql-5.1.0.html#p3misc) function.
 - In MySQL, you can use the [`ST_AsGeoJSON()`](https://dev.mysql.com/doc/refman/8.0/en/spatial-geojson-functions.html#function_st-asgeojson) function.
