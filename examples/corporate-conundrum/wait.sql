-- Redirect to the next question when all players have answered
set page_params = json_object('game_id', $game_id, 'player', $player);
select CASE
        (SELECT count(*) FROM answers WHERE question_id = $question_id AND game_id = CAST($game_id AS INTEGER))
        WHEN (SELECT count(*) FROM players WHERE game_id = CAST($game_id AS INTEGER))
        THEN '0; ' || sqlpage.link('next-question.sql', $page_params)
        ELSE 3
    END as refresh,
    sqlpage_shell.*
FROM sqlpage_shell;

-- Insert the answer into the answers table
INSERT INTO answers(game_id, player_name, question_id, answer_value)
SELECT CAST($game_id AS INTEGER) as game_id,
    $player as player_name,
    CAST($question_id AS INTEGER) as question_id,
    CAST($answer AS INTEGER) as answer_value
WHERE $answer IS NOT NULL;
-- Redirect to the next question
SELECT 'text' as component,
    'Waiting for other players to answer... The following players still have not answered: ' as contents;
select group_concat(name, ', ') as contents,
    TRUE as bold
from players
where game_id = CAST($game_id AS INTEGER)
    and not EXISTS (
        SELECT 1
        FROM answers
        WHERE answers.game_id = CAST($game_id AS INTEGER)
            AND answers.player_name = players.name
            AND answers.question_id = CAST($question_id AS INTEGER)
    );
