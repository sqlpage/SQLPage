-- Request values are bound parameters, never interpolated into SQL.
set valid = sqlpage.request_method() = 'POST'
    and length(trim(:name)) between 1 and 80
    and length(coalesce(:team, '')) <= 160;
update profiles
set name = trim(:name), team = trim(coalesce(:team, ''))
where id = 1 and $valid;
select 'redirect' as component,
    '/landing-demos/demo.sql?component=form&saved=' || case when $valid then '1' else '0' end as link;
