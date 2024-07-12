-- We need to redirect the user to the next question in the game if there is one, or to the game over page if there are no more questions.
with next_question as (
    SELECT 
        'question.sql' as page,
        json_object(
            'game_id', $game_id,
            'question_id', game_questions.question_id,
            'player', $player
        ) as params
    FROM game_questions
    WHERE game_id = $game_id::integer
        AND NOT EXISTS (
            -- This will filter out questions that have already been answered by the player
            SELECT 1
            FROM answers
            WHERE answers.game_id = game_questions.game_id
                AND answers.player_name = $player
                AND answers.question_id = game_questions.question_id
        )
    ORDER BY game_order
    LIMIT 1
),
next_page as (
        SELECT * FROM next_question
    UNION ALL
        SELECT 'game-over.sql' as page, json_object('game_id', $game_id) as params
        WHERE NOT EXISTS (SELECT 1 FROM next_question)
)
SELECT 'redirect' as component,
    sqlpage.link(page, params) as link
FROM next_page
LIMIT 1;