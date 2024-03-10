-- Checks that we can have a page with a single dynamic component containing multiple children
select 'dynamic' as component,
    '[
        {"component":"dynamic", "properties": [
            {"component":"text"},
            {"contents":"It works !", "bold":true}
        ]}
    ]' as properties;