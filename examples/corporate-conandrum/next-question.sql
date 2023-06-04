SELECT 'http_header' as component,
    COALESCE(
        (
            SELECT 'question.sql?game_id=' || $game_id || '&question_id=' || game_questions.question_id || '&player=' || $player as "Location"
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
        'game-over.sql?game_id=' || $game_id
    ) as 'Location';
SELECT 'text' as component,
    'redirecting to next question...' as contents;