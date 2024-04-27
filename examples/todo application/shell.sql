select 'shell' as component,
    printf('Todo list (%d)', count(*)) as title,
    'timeline' as menu_item
from todos;