-- Checks that we can have a page with a single dynamic component containing multiple children
select 'dynamic' as component,
    '[
        {"component":"shell", "title":"It works !"},
        {"component":"text"},
        {"contents":"Yes it does !", "bold":true}
    ]' as properties;