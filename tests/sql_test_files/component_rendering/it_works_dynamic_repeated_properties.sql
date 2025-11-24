-- Checks that we can have a page with a single dynamic component containing multiple children
select 'dynamic' as component,
    '{"component":"text"}' as properties,
    '{"contents":"Hello, ", "bold":true}' as properties,
    '{"component":"text"}' as properties,
    '{"contents":"It works !", "bold":true}' as properties;
