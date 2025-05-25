-- This test checks that the size of the form field can successfully roundtrip,
-- from POST variable to sqlpage variable to handlebars, back to the client
set x = :x;
select 'text' as component, $x as contents;
