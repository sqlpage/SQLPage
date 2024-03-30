-- important: we do not accept file uploads from unauthenticated users
select 'redirect' as component, '/login.sql' as link -- redirect to the login page if the user is not logged in
where not exists (
    select true from session
    where
        sqlpage.cookie('session_token') = id and
        created_at > datetime('now', '-1 day') -- require the user to log in again after 1 day
);

-- Redirect the user back to the form if no file was uploaded
select 'redirect' as component, '/upload_form.sql' as link
where sqlpage.uploaded_file_mime_type('Image') NOT LIKE 'image/%';

insert or ignore into image (title, description, image_url)
values (
    :Title,
    :Description,
    -- Persist the uploaded file to the local "images" folder at the root of the website and return the path
    sqlpage.persist_uploaded_file('Image', 'images', 'jpg,jpeg,png,gif')
    -- alternatively, if the images are small, you could store them in the database directly with the following line
    -- sqlpage.read_file_as_data_url(sqlpage.uploaded_file_path('Image'))
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