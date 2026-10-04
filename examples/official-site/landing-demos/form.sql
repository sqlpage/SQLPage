select 'form' as component, 'Say hello' as title, 'Submit' as validate;
select 'name' as name, 'Your name' as label, name as value,
    true as required, 6 as width
from profiles where id = 1;
select 'team' as name, 'Your team' as label, team as value, 6 as width
from profiles where id = 1;
