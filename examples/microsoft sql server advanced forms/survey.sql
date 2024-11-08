SELECT 'form' as component, 'Survey' as title;
SELECT id as name, question_text as label, 'textarea' as type
FROM questions;

-- Save all the answers to the database, whatever the number and id of the questions
INSERT INTO survey_answers (question_id, answer)
SELECT
    question_id,
    json_unquote(
        json_extract(
            sqlpage.variables('post'),
            concat('$."', question_id, '"')
        )
    )
FROM json_table(
    json_keys(sqlpage.variables('post')),
    '$[*]' columns (question_id int path '$')
) as question_ids;

-- Show the answers
select 'card' as component, 'Survey results' as title;
select
    questions.question_text as title,
    survey_answers.answer as description,
    'On ' || survey_answers.timestamp as footer
from survey_answers
inner join questions on questions.id = survey_answers.question_id;
