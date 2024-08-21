select 'shell' as component,
    printf('Todo list (%d)', count(*)) as title,
    'timeline' as menu_item,
    TRUE as sidebar,
    'dark' as theme,
    './sqlpage.css' as css
from todos;