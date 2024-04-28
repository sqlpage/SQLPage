select 'redirect' as component, '/login.sql' as link
where not exists(select * from user_sessions where session_id = sqlpage.cookie('session_id'));


select 'card' as component, 'My secure protected page' as title, 1 as columns;
select 
    'Secret video' as title,
    'https://www.youtube.com/embed/mXdgmSdaXkg' as embed,
    'accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share' as allow,
    'iframe'         as embed_mode,
    '700'            as height;