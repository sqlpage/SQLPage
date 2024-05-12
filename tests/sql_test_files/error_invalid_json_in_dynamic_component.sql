select 'dynamic' as component,
    '{ "this object": "has a forbidden comma" , }' as properties; -- this is invalid JSON, and should cause an error