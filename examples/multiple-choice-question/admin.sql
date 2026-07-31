select 'dynamic' as component, sqlpage.read_file_as_text('website_header.json') as properties;

-- Successful actions are transient feedback, so they use toasts instead of occupying page content.
select 'toast' as component, 'Saved' as title, 'green' as color, 'check' as icon where $saved is not null;
select 'toast' as component, 'Deleted' as title, 'green' as color, 'trash' as icon where $deleted is not null;
-- A failed deletion needs the user's attention and remains an inline alert.
select 'alert' as component, 'This option cannot be deleted' as title, 'danger' as color, 'If an option has already been chosen by at least one respondant, then it cannot be deleted' as description where $cannot_delete is not null;

select 'dynamic' as component,
    json_array(
        json_object(
            'component', 'form',
            'title', CONCAT('Option ', id),
            'action', CONCAT('edit_option.sql?id=', id),
            'validate', '',
            'id', CONCAT('option', id)
        ),
        json_object(
            'type', 'text',
            'name', 'profile_description',
            'label', 'Profile description',
            'value', profile_description
        ),
        json_object(
            'type', 'number',
            'name', 'score',
            'min', 0,
            'label', 'Score',
            'value', score
        ),
        json_object('component', 'button', 'size', 'sm'),
        json_object('title', 'Delete', 'outline', 'danger', 'icon', 'trash', 'link', CONCAT('delete_option.sql?id=', id)),
        json_object('title', 'Save', 'outline', 'success', 'icon', 'device-floppy', 'form', CONCAT('option', id))
    ) as properties
from dog_lover_profiles;

select 'button' as component, 'center' as justify;
select 'Create new question' as title, 'create_question.sql' as link;
