insert into dog_lover_profiles(profile_description, score) values ('', 50)
returning
    'redirect' as component,
    'admin.sql' as link;