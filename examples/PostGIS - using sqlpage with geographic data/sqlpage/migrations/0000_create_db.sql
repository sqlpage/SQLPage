CREATE EXTENSION  IF NOT EXISTS postgis;

-- Create a table with a postgis geometry column
CREATE TABLE IF NOT EXISTS spatial_data (
    id serial primary key NOT NULL,
    title text NOT NULL,
    geom geometry NULL,
    description text NOT NULL,
    created_at timestamp without time zone NULL DEFAULT CURRENT_TIMESTAMP
);


CREATE OR REPLACE VIEW distances AS
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