select 'dynamic' as component, sqlpage.read_file_as_text('website_header.json') as properties;

SELECT
  'form' AS component,
  'What dog lover are you ?' AS title,
  'process.sql' AS action;

select 'radio' as type, 'profile' as name, id as value, profile_description as label
from dog_lover_profiles;