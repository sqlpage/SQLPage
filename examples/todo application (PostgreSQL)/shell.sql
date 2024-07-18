select
    'shell' as component,
    format ('Todo list (%s)', count(*)) as title,
    'timeline' as menu_item
from
    todos;