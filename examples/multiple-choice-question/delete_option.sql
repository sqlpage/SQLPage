select 'redirect' as component, 'admin.sql?cannot_delete' as link
where exists (select 1 from answers where profile_id = $id);

delete from dog_lover_profiles where id = $id
returning
    'redirect' as component,
    'admin.sql?deleted' as link;