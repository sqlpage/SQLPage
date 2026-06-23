SELECT 'form' as component, 'Survey' as title;
SELECT id as name, question_text as label, 'textarea' as type
FROM questions;

-- Save all the answers to the database, whatever the number and id of the questions
INSERT INTO survey_answers (question_id, answer)
SELECT
    TRY_CONVERT(int, answers.[key]) as question_id,
    answers.value as answer
FROM OPENJSON(sqlpage.variables('post')) as answers
WHERE TRY_CONVERT(int, answers.[key]) IS NOT NULL;

-- Show the answers
select 'card' as component, 'Survey results' as title;
select
    questions.question_text as title,
    survey_answers.answer as description,
    'On ' + CONVERT(varchar(33), survey_answers.timestamp, 126) as footer
from survey_answers
inner join questions on questions.id = survey_answers.question_id;
