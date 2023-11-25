-- important: we do not accept file uploads from unauthenticated users
select 'redirect' as component, '/login.sql' as link -- redirect to the login page if the user is not logged in
where not exists (
    select true from session
    where
        sqlpage.cookie('session_token') = id and
        created_at > datetime('now', '-1 day') -- require the user to log in again after 1 day
);

insert or ignore into image (title, description, image_url)
values (
    :Title,
    :Description,
    sqlpage.read_file_as_data_url(sqlpage.uploaded_file_path('Image'))
)
returning 'redirect' as component,
          format('/?created_id=%d', id) as link;

-- If the insert failed, warn the user
select 'alert' as component,
    'red' as color,
    'alert-triangle' as icon,
    'Failed to upload image' as title,
    'Please try again with a smaller picture. Maximum allowed file size is 500Kb.' as description
;