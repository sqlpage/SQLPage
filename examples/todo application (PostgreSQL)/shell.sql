select
    'shell' as component,
    format ('Todo list (%s)', count(*)) as title,
    'batch' as menu_item,
    'timeline' as menu_item
from
    todos;