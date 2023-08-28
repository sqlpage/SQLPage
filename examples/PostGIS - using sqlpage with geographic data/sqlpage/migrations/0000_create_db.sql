CREATE EXTENSION  IF NOT EXISTS postgis;

-- Create a spatialite-enabled database
CREATE TABLE spatial_data (
    id SERIAL,
    title TEXT NOT NULL,
    geom geometry,
    description TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);


CREATE VIEW distances AS
SELECT from_point.id AS from_id,
    from_point.title AS from_label,
    to_point.id AS to_id,
    to_point.title AS to_label,
    ST_Distance(
        from_point.geom,
        to_point.geom,
        TRUE
    ) AS distance
FROM spatial_data AS from_point, spatial_data AS to_point
WHERE from_point.id != to_point.id;