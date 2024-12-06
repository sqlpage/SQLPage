-- This demonstrates how to build multi-step forms using the `form` component, and hidden inputs.
select 'dynamic' as component, properties FROM example WHERE component = 'shell' LIMIT 1;

select 'text' as component, '
# SQLPage Multi-Step Form Example

The example below demonstrates how to build complex multi-step forms using the [`form`](/documentation.sql?component=form#component) component.
In this example, the list of cities is taken from a dynamic database table,
and each step of the form is shown conditionally based on the previous step.
The form has a variable number of fields: after the number of adults and children are selected,
a field is shown for each passenger to enter their name.

See [the SQL source on GitHub](https://github.com/sqlpage/SQLPage/blob/main/examples/official-site/examples/multistep-form) for the full code.
' as contents_md;

create temporary table if not exists cities as
select 1 as id, 'Cairo' as city union all
select 2, 'Delhi' union all
select 3, 'Dhaka' union all
select 4, 'Istanbul' union all
select 5, 'Karachi' union all
select 6, 'Kinshasa' union all
select 7, 'Lagos' union all
select 8, 'Mexico' union all
select 9, 'New York City' union all
select 10, 'Paris';

select 'form' as component, 'Book a flight' as title,
    case when :adults is null 
        then 'Next'
        else 'Book the flight !'
    end as validate,
    case when :adults is null
        then ''
        else 'result.sql'
    end as action;

-- First step: Select origin city
select 'select' as type, 'origin' as name, 'From' as label, 'Select your origin city' as placeholder, true as searchable,
case when :origin is null then 12 else 6 end as width, -- The origin field takes the entire width of the form, unless it's already selected
CAST(:origin AS INTEGER) as value, -- We keep the value of the origin city in the form. All form fields are submitted as text
json_group_array(json_object('value', id, 'label', city)) as options
from cities;

-- Second step: Select destination city, but only show destinations once origin is selected, and not the same as origin
select 'select' as type, 'destination' as name, 'To' as label, 'Select your destination city' as placeholder, true as searchable,
6 as width, -- The destination field always takes half the width of the form
CAST(:destination AS INTEGER) as value, -- We keep the value of the destination city in the form
json_group_array(json_object('value', id, 'label', city)) as options
from cities
where id != CAST(:origin AS INTEGER) -- We can't fly to the same city we're flying from
having :origin is not null; -- Only show destinations once origin is selected

-- Third step: Select departure date and number of passengers
select 'date' as type, 'departure_date' as name, 'Departure date' as label, 'When do you want to depart?' as placeholder,
date('now') as min, date('now', '+6 months') as max, 
4 as width, -- The departure date field takes a third of the width of the form
coalesce(:departure_date, date('now', '+7 days')) as value -- Default to a week from now
where :destination is not null; -- Only show departure date once destination is selected

select 'number' as type, 'adults' as name, 'Number of Adults' as label, 'How many adults are flying?' as placeholder, 1 as min, 10 as max,
coalesce(:adults, 2) as value, -- Default to 1 adult
:adults is not null as readonly, -- The number of adults field is readonly once it's selected
4 as width -- The number of adults field takes a third of the width of the form
where :destination is not null; -- Only show number of adults once destination is selected

select 'number' as type, 'children' as name, 'Number of Children' as label, 'How many children are flying?' as placeholder,
coalesce(:children, '0') as value, -- Default to 0 children
:children is not null as readonly, -- The number of adults field is readonly once it's selected
4 as width -- The number of children field takes a third of the width of the form
where :destination is not null; -- Only show number of children once destination is selected

-- Fourth step: Enter passenger details
with recursive passenger_ids as (
    select 0 as id, 0 as passenger_type union all
    select id + 1 as id,
        case when id < CAST(:adults AS INTEGER) 
            then 'adult'
            else 'child'
        end as passenger_type
    from passenger_ids
    where id < CAST(:adults AS INTEGER) + CAST(:children AS INTEGER)
)
select 'text' as type,
    printf('%s_names[]', passenger_type) as name,
    true as required,
    printf('Passenger %d (%s)', id, passenger_type) as label,
    printf('Enter %s passenger name', passenger_type) as placeholder
from passenger_ids
where id>0 and :adults is not null -- Only show passenger details once number of adults and children are selected