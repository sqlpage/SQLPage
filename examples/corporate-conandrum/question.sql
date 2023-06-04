select 'shell' as component, * FROM shell;

SELECT 'form' as component,
    question_text as title,
    'Submit your answer' as validate,
    'wait.sql?game_id='|| $game_id ||'&question_id=' || $question_id ||'&player=' || $player as action
FROM questions
where id = $question_id::integer;

SELECT 'answer' as name,
    CASE
    $player
        WHEN impostor THEN 'Try to trick the other players into answering: ' || wrong_answer || ', but try to make your own answer correct.'
        ELSE 'Discuss the question with the other players and then submit your answer'
    END as label,
    'Your answer' as placeholder,
    'number' as type,
    TRUE as required,
    TRUE as autofocus,
    0 as min
FROM game_questions
WHERE game_id = $game_id::integer
    AND question_id = $question_id::integer;

SELECT 'alert' as component,
    'red' as color,
    'Make them guess: ' || wrong_answer as title,
    'You are the impostor!
    Your goal is to sabotage the game by making others give an answer that will be closer to ' || wrong_answer || ' then to the true answer.
    The more other players you manage to trick, the more points you will get.' as description
FROM game_questions
WHERE game_id = $game_id::integer
    AND impostor = $player
    AND question_id = $question_id::integer;