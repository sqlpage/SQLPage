---
rows:
  - component: map
    title: Paris
    zoom: 13
    latitude: 48.85
    longitude: 2.34
    tile_source: https://api.maptiler.com/maps/streets-v2/{z}/{x}/{y}.png?key=RwoF6Y3gcKx4OBMbvqOY
  - title: Notre Dame
    icon: building-castle
    color: indigo
    latitude: 48.853
    longitude: 2.3498
    description_md: A beautiful cathedral.
    link: https://en.wikipedia.org/wiki/Notre-Dame_de_Paris
  - title: Eiffel Tower
    icon: tower
    color: red
    latitude: 48.8584
    longitude: 2.2945
    description_md: A tall tower. [Wikipedia](https://en.wikipedia.org/wiki/Eiffel_Tower)
  - title: Tower to Cathedral
    geojson:
      type: LineString
      coordinates:
        - - 2.2945
          - 48.8584
        - - 2.3498
          - 48.853
    color: teal
    description: A nice 45 minutes walk.
---

### Maps with links and rich descriptions

Demonstrates how to create an engaging map with custom icons, colors, rich descriptions with markdown support, and connecting points with lines.
Perfect for visualizing multi-dimensional relationships between points on a map, like routes between locations.

Note that the map tile source is set to a MapTiler map. The API key included in the URL in this demo will not work on your own website.
You should get your own API key at [MapTiler](https://www.maptiler.com/cloud/).
