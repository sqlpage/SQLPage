select 'redirect' as component, '/login.sql' as link -- redirect to the login page if the user is not logged in
where not exists (select true from session where sqlpage.cookie('session_token') = id and created_at > datetime('now', '-1 day')); -- require the user to log in again after 1 day

select 'form' as component, 'Upload a new image' as title, 'upload.sql' as action;
select 'text' as type, 'Title' as name, true as required;
select 'text' as type, 'Description' as name;
select 'file' as type, 'Image' as name, 'image/*' as accept;