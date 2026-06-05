select
    'text' as component,
    true as article,
    CONCAT('
# Welcome to my website

Using SQLPage v', sqlpage.version(), '

Connected to **MySQL** v', version ()) as contents_md;
