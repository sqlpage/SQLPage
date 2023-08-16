INSERT INTO spatial_data (title, geom, description)
VALUES (
    :Title,
    MakePoint(
        CAST(:Longitude AS REAL),
        CAST(:Latitude AS REAL
    ), 4326),
    :Text
) RETURNING 
    'redirect' AS component,
    'index.sql' AS link;