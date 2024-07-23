insert into sqlpage_files (path, contents)
values (:path, :contents::bytea)
on conflict (path)
do update set contents = excluded.contents
returning 
    'redirect' as component,
    sqlpage.link(
        'index.sql',
        json_build_object('inserted', :path)
    ) as link;