select 'shell' as component,
    *
FROM shell;
-- Display the list of players with a link for each one to start playing
INSERT INTO players(name, game_id)
SELECT $Player as name,
    $id::integer as game_id
WHERE $Player IS NOT NULL;
SELECT 'list' as component,
    'Players' as title;
SELECT name as title,
    'next-question.sql?game_id=' || game_id || '&player=' || name as link
FROM players
WHERE game_id = $id::integer;
---------------------------
-- Player insertion form --
---------------------------
SELECT 'form' as component,
    'Add a new player' as title,
    'Add to game' as validate;
SELECT 'Player' as name,
    'Player name' as placeholder,
    TRUE as autofocus;
-- Insert a new question into the game_questions table for each new player
INSERT INTO game_questions(
        game_id,
        question_id,
        wrong_answer,
        impostor,
        game_order
    )
SELECT $id::integer as game_id,
    questions.id as question_id,
    -- When the true answer is small, set the wrong answer to just +/- 1, otherwise -25%/+75%.
    -- When it is a date between 1200 and 2100, make it -25 % or +75 % of the distance to today
    CAST(CASE
        WHEN questions.true_answer < 10 THEN questions.true_answer + 1 - 2 * abs(random() %2) -- wrong answer = true answer +/- 1
        WHEN questions.true_answer > 1200
        AND questions.true_answer < 2100 THEN 2023 - (2023 - questions.true_answer) * (abs(random() %2) + 0.75) -- wrong answer = true answer -25% or +75% of the distance to today
        ELSE questions.true_answer * (abs(random() %2) + 0.75)
    END AS INTEGER) as wrong_answer,
    -- wrong answer = true answer +/- 50% TODO: better wrong answer generation
    $Player as impostor,
    random() as game_order
FROM questions
    LEFT JOIN game_questions ON questions.id = game_questions.question_id
    AND game_questions.game_id = $id::integer
WHERE game_questions.question_id IS NULL
    AND $Player IS NOT NULL
ORDER BY random()
LIMIT 1;