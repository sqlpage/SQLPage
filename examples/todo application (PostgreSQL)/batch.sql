-- Include 'shell.sql' to generate the page header and footer
select 'dynamic' as component, sqlpage.run_sql('shell.sql') as properties;

-- Define a Common Table Expression (CTE) named 'updated'
-- CTEs are temporary named result sets, useful for complex queries
-- Here, it's used to perform the update and capture the results in one step
with updated as (
    -- Update the 'todos' table and return the modified rows
    -- This approach allows us to both update the data and use it for reporting
    update todos set
    -- Modify the title based on user input for labels
    -- The CASE statements handle different scenarios for label management
    title = case 
        -- If :remove_label is null, we keep the existing title as is
        when :remove_label is null then
            title
        else  
            -- Remove any existing labels (text within parentheses)
            -- This uses a regular expression to strip out (label) from the end
            regexp_replace(title, '\s*\(.*\)', '')
        end
        -- Concatenate the result with a new label if provided
        ||
        case 
            -- If no new label is provided, we don't add anything
            when :new_label is null or :new_label = '' then
                ''
            else
                -- Add the new label in parentheses at the end
                ' (' || :new_label || ')'
        end
    -- Determine which todos to update based on user selection
    where 
        -- Update specific todos if their IDs are in the :todos parameter
        -- :todos is a JSON array of todo string IDs, e.g. ["1", "2", "3"]
        -- that optionally includes "all" to update all todos
        id in (
            -- Parse the JSON array of todo IDs and convert each to integer
            -- This allows for multiple todo selection in the UI
            select e::int from jsonb_array_elements_text(:todos::jsonb) e 
            where e != 'all'
        )
        -- If 'all' is the only selected, update every todo (by making the where condition always true)
        or :todos = '["all"]'
    -- Return all updated rows for counting and potential further use
    returning *
)
-- Generate an alert component to inform the user about the update result
-- This provides immediate feedback on the operation's outcome
select 'alert' as component,
    'Batch update' as title,    
    -- Create a dynamic message with the count of updated todos
    format('%s todos updated', (select count(*) from updated)) as description
-- Only display the alert if at least one todo was updated
-- This prevents showing unnecessary alerts for no-op updates
where exists (select * from updated);

-- Create a form component for the batch update interface
-- This sets up the structure for the user input form
select 'form' as component,
    'Batch update' as title,
    'Update all todos' as contents;

-- Create a select input for choosing which todos to update
-- This allows users to pick multiple todos or all todos for updating
select 
    'select' as type,
    'Update these todos' as label,
    'todos[]' as name,
    true as multiple,
    true as dropdown,
    true as required,
    -- Combine a static "all" option with dynamic options for each todo
    -- This uses JSON functions to build a complex data structure for the UI
    -- The JSON structure is used to set the label, value, and selection state for each option
    -- The generated JSON looks like this:
    -- [{"label":"Update all todos","value":"all","selected":true},{"label":"Todo 1","value":"1","selected":false}]
    jsonb_build_array(jsonb_build_object( -- json_build_object takes a list of key-value pairs and returns a JSON object
        'label', 'Update all todos', -- The label of the option
        'value', 'all', -- The value of the option
        'selected', :todos = '["all"]' or :todos is null -- Pre-select 'all' only if it was previously chosen or if :todos is not set (the page was just loaded)
    )) ||
    -- Generate an option for each todo in the database
    jsonb_agg(jsonb_build_object(
        'label', title,
        'value', id,
        -- Pre-select this todo if it was in the previous selection
        'selected', (id in (select e::int from jsonb_array_elements_text(:todos::jsonb) e where e != 'all'))
    )) as options
from todos;

-- Create a text input for entering a new label
-- This allows users to specify the label to be added to the selected todos
select 'new_label' as name, 'New label' as label;

-- Create a checkbox for optionally removing existing labels
-- This gives users the choice to strip old labels before adding a new one
select 'checkbox' as type, 'Remove previous labels' as label, 'remove_label' as name;