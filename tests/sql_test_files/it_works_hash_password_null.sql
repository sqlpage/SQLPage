SELECT 'text' as component, 
    case when sqlpage.hash_password(null) is null then 'It works !' else 'Error !' end
    as contents;
