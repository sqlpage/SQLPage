select 'shell' as component,
    'My image gallery' as title,
    (
        case when sqlpage.cookie('session_token') is null then 'login'
        else 'logout' end
    ) as menu_item;

select 'card' as component,
    'My image gallery' as title;

select title, description, image_url as top_image
from image;

select 'Your gallery is empty' as title,
    'You have not uploaded any images yet. Click the button below to upload a new image.' as description
where not exists (select 1 from image);

select 'button' as component;
select
    'Upload a new image' as title,
    'upload_form.sql' as link,
    'plus' as icon,
    'primary' as color;
