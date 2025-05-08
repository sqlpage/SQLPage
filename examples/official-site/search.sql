set search = nullif(trim($search), '');

-- Check for exact matches and redirect if found
set redirect = CASE 
    WHEN EXISTS (SELECT 1 FROM component WHERE name = $search) THEN sqlpage.link('/component.sql', json_object('component', $search))
    WHEN EXISTS (SELECT 1 FROM sqlpage_functions WHERE name = $search) THEN sqlpage.link('/functions.sql', json_object('function', $search))
END
SELECT 'redirect' as component, $redirect as link WHERE $redirect IS NOT NULL;

select 'dynamic' as component, json_patch(json_extract(properties, '$[0]'), json_object(
    'title', coalesce($search || ' | ', '') || 'SQLPage documentation search'
)) as properties
FROM example WHERE component = 'shell' LIMIT 1;

SELECT 'form' as component,
    'GET' as method,
    true as auto_submit,
    'Search documentation' as title;

SELECT 'text' as type,
    'search' as name,
    '' as label,
    true as autofocus,
    'Search for components, parameters, functions...' as placeholder,
    $search as value;

SELECT 'text' as component,
    CASE 
        WHEN $search IS NULL THEN 'Enter a search term above to find documentation about components, parameters, functions, and blog posts.'
        WHEN NOT EXISTS (
            SELECT 1 FROM documentation_fts 
            WHERE documentation_fts = $search
        ) THEN 'No results found for "' || $search || '".'
        ELSE NULL
    END as contents;

SELECT 'list' as component,
    'Search Results' as title,
    'No results found for "' || $search || '".' as empty_description
WHERE $search IS NOT NULL;

WITH search_results AS (
    SELECT 
        COALESCE(
              component_name || ' component: parameter ' || parameter_name
            , component_name || ' component' || IF(component_example_description IS NULL, '', ' example')
            , 'blog: ' || blog_title
            , 'sqlpage.' || function_name || '(...' || function_parameter_name || '...)'
            , 'sqlpage.' || function_name || '(...)'
        ) as title,
        COALESCE(
            component_description,
            parameter_description,
            blog_description,
            function_parameter_description,
            function_description,
            component_example_description
        ) as description,
        CASE 
            WHEN component_name IS NOT NULL THEN 
                json_object(
                    'page', '/component.sql',
                    'parameters', json_object('component', component_name)
                )
            WHEN parameter_name IS NOT NULL THEN 
                json_object(
                    'page', '/component.sql',
                    'parameters', json_object('component', (
                        SELECT component 
                        FROM parameter 
                        WHERE name = parameter_name 
                        LIMIT 1
                    ))
                )
            WHEN blog_title IS NOT NULL THEN 
                json_object(
                    'page', '/blog.sql',
                    'parameters', json_object('post', blog_title)
                )
            WHEN function_name IS NOT NULL THEN 
                json_object(
                    'page', '/functions.sql',
                    'parameters', json_object('function', function_name)
                )
            WHEN function_parameter_name IS NOT NULL THEN 
                json_object(
                    'page', '/functions.sql',
                    'parameters', json_object('function', (
                        SELECT function 
                        FROM sqlpage_function_parameters 
                        WHERE name = function_parameter_name 
                        LIMIT 1
                    ))
                )
        END as link_data,
        rank
    FROM documentation_fts
    WHERE $search IS NOT NULL 
    AND documentation_fts = $search
)
SELECT 
    max(title) as title,
    max(description) as description,
    sqlpage.link(link_data->>'page', link_data->'parameters') as link
FROM search_results
GROUP BY link_data
ORDER BY 
    rank,
    CASE 
        WHEN title LIKE 'component:%' THEN 1
        WHEN title LIKE 'parameter:%' THEN 2
        WHEN title LIKE 'blog:%' THEN 3
        WHEN title LIKE 'function:%' THEN 4
    END,
    description;
