select
    'dynamic' as component,
    sqlpage.run_sql ('shell.sql') as properties;

-- this does the same thing as index.sql, but uses the normal form component instead of our fancy dual-list component
select
    'form' as component,
    'form_action.sql' as action;

select
    'select' as type,
    true as searchable,
    true as multiple,
    'selected_items[]' as name,
    'Users in this group' as label,
    -- JSON_MERGE combines two JSON documents:
    -- 1. A JSON object with an empty label
    -- 2. An array of user objects created by JSON_ARRAYAGG
    JSON_MERGE (
        -- Creates a simple JSON object with a single empty property {"label": ""}
        JSON_OBJECT ('label', ''),
        -- JSON_ARRAYAGG takes multiple rows and combines them into a JSON array
        -- Each element in the array is a JSON object created by json_object()
        JSON_ARRAYAGG (
            -- Creates a JSON object for each user with:
            -- - {"label": "the user's name", "value": "the user's ID", "selected": true } (if the user is in the group)
            json_object (
                'label',
                users.name,
                'value',
                users.id,
                'selected',
                group_members.group_id is not null -- the left join creates NULLs for users not in the group
            )
        )
    ) as options
from
    users
    left join group_members on users.id = group_members.user_id
    and group_members.group_id = 1;