select 'dynamic' as component, sqlpage.read_file_as_text('website_header.json') as properties;

select timestamp, profile_description, score from answers
inner join dog_lover_profiles on dog_lover_profiles.id = answers.profile_id;

select 'csv' as component;
select * from answers;