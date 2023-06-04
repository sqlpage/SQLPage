select 'shell' as component, * FROM shell;

INSERT INTO players(name, game_id)
SELECT $Player as name,
    $id::integer as game_id
WHERE $Player IS NOT NULL;

SELECT 'list' as component, 'Players' as title;
SELECT 
    name as title,
    'next-question.sql?game_id=' || game_id || '&player=' || name as link
FROM players WHERE game_id = $id::integer;

SELECT 'form' as component, 'Add a new player' as title, 'Add to game' as validate;
SELECT 'Player' as name, 'Player name' as placeholder, TRUE as autofocus;

-- Insert a new question into the game_questions table for each new player
INSERT INTO game_questions(game_id, question_id, wrong_answer, impostor, game_order)
SELECT
     $id::integer as game_id,
     questions.id as question_id,
     questions.true_answer * (abs(random() %2) + 0.5) as wrong_answer, -- wrong answer = true answer +/- 50% TODO: better wrong answer generation
     $Player as impostor,
     random() as game_order
FROM questions
LEFT JOIN game_questions ON questions.id = game_questions.question_id AND game_questions.game_id = $id::integer
WHERE game_questions.question_id IS NULL AND $Player IS NOT NULL
ORDER BY random()
LIMIT 1;