-- Sorting this column reloads the current page with sort_username=ASCENDING
-- or sort_username=DESCENDING. The query below applies that choice before
-- rendering the table, so sorting remains correct when the result is paginated.
select 'table' as component,
    'username' as server_sort_column,
    'action' as markdown;
select *,
    format('[Edit](edit.sql?id=%s)', id) as action
from users
order by
    case when $sort_username = 'ASCENDING' then username end asc,
    case when $sort_username = 'DESCENDING' then username end desc,
    id;
