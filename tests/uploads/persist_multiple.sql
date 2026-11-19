select 'json' as component,
    coalesce(sqlpage.persist_uploaded_files('photos', 'tests/uploads/persisted_multiple', 'txt'), 'null') as contents;
