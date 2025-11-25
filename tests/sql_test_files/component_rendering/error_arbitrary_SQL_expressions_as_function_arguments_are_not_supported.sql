select 'shell' as component, 
    lower(sqlpage.url_encode(lower('HELLO'))) as title;
-- this is invalid, because the sqlpage pseudo-function is sandwiched between two native SQL functions.
-- It can't be executed neither before nor after the query is executed.