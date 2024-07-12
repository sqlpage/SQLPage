-- Redirect to the next question when all players have answered
set page_params = json_object('game_id', $game_id, 'player', $player);
select CASE
        (SELECT count(*) FROM answers WHERE question_id = $question_id AND game_id = $game_id::integer)
        WHEN (SELECT count(*) FROM players WHERE game_id = $game_id::integer)
        THEN '0; ' || sqlpage.link('next-question.sql', $page_params)
        ELSE 3
    END as refresh,
    sqlpage_shell.*
FROM sqlpage_shell;

-- Insert the answer into the answers table
INSERT INTO answers(game_id, player_name, question_id, answer_value)
SELECT $game_id::integer as game_id,
    $player as player_name,
    $question_id::integer as question_id,
    $answer::integer as answer_value
WHERE $answer IS NOT NULL;
-- Redirect to the next question
SELECT 'text' as component,
    'Waiting for other players to answer... The following players still have not answered: ' as contents;
select group_concat(name, ', ') as contents,
    TRUE as bold
from players
where game_id = $game_id::integer
    and not EXISTS (
        SELECT 1
        FROM answers
        WHERE answers.game_id = $game_id::integer
            AND answers.player_name = players.name
            AND answers.question_id = $question_id::integer
    );