insert into answers(profile_id)
select CAST(:profile as integer)
where :profile is not null
returning 'redirect' as component, 'results.sql' as link;