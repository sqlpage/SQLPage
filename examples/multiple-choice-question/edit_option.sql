update dog_lover_profiles
set profile_description = :profile_description, score = :score
where id = $id
returning
    'redirect' as component,
    'admin.sql?saved' as link;