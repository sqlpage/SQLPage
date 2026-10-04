select 'form' as component, 'Your profile' as title,
    'Save profile' as validate, 'device-floppy' as validate_icon,
    '/landing-demos/save.sql' as action, 'landing-profile' as id;
select 'name' as name, 'Your name' as label, name as value,
    'Ada Lovelace' as placeholder, 'user' as prefix_icon,
    true as required, 80 as maxlength, 6 as width
from profiles where id = 1;
select 'team' as name, 'Your team' as label, team as value,
    'Small team, serious tools' as placeholder, 'users' as prefix_icon,
    'Edits update this shared demo profile.' as description,
    160 as maxlength, 6 as width
from profiles where id = 1;
