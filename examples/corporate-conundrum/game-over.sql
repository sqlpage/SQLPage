select * FROM sqlpage_shell;

-- Game over, questions breakdown
select 'card' as component,
    2 AS columns,
    'The game is over' as title,
    'Breaking down the results per question' as description;
SELECT question_text as title,
    'The true answer was ' || true_answer || '. ' || impostor || ' tried to make everyone think it was ' || wrong_answer || '. ' || (
        select group_concat(
                player_name || ' voted ' || answer_value || CASE
                    WHEN abs(answer_value - true_answer) < abs(answer_value - wrong_answer)
                    THEN ' and earned a point'
                    WHEN player_name = impostor
                    THEN ' and did not earn a point'
                    ELSE ' and gave a point to ' || impostor
                END,
                ', '
            )
        from answers
        where answers.game_id = $game_id::integer
            and answers.question_id = questions.id
    ) || '.' as description,
    explanation as footer
FROM game_questions
    JOIN questions ON game_questions.question_id = questions.id
WHERE game_id = $game_id::integer
ORDER BY game_order;

-- Point count
SELECT 'chart' as component,
    'Scores' as title,
    'bar' as type;
SELECT name as label,
    (
        select sum(
                (players.name = player_name AND NOT impostor_won)
                OR (players.name != player_name AND players.name = impostor AND impostor_won)
            )
        FROM game_results WHERE game_id = $game_id
    ) as value
FROM players
WHERE game_id = $game_id::integer
ORDER BY value DESC;