INSERT INTO spatial_data (label, geom)
VALUES (
    :Label,
    MakePoint(
        CAST(:Longitude AS REAL),
        CAST(:Latitude AS REAL
    ), 4326)
) RETURNING 
    'redirect' AS component,
    'index.sql' AS link;