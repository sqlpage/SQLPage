CREATE TABLE games (
    id INTEGER PRIMARY KEY,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE players (
    name TEXT NOT NULL,
    game_id INTEGER NOT NULL,
    PRIMARY KEY (name, game_id),
    FOREIGN KEY (game_id) REFERENCES games(id)
);

CREATE TABLE questions (
    id INTEGER PRIMARY KEY,
    question_text TEXT,
    category CHAR(4),
    explanation TEXT,
    true_answer INTEGER
);

CREATE TABLE game_questions (
    game_id INTEGER,
    question_id INTEGER,
    wrong_answer INTEGER, -- the wrong answer that the impostor will try to convince others is correct
    impostor TEXT, -- the player who will receive the wrong answer
    game_order INTEGER, -- indicates the order in which the questions are asked
    PRIMARY KEY (game_id, question_id),
    FOREIGN KEY (question_id) REFERENCES questions(id),
    FOREIGN KEY (game_id, impostor) REFERENCES players(game_id, name)
);

CREATE TABLE answers (
    game_id INTEGER,
    player_name TEXT,
    question_id INTEGER,
    answer_value INTEGER,
    PRIMARY KEY (game_id, question_id, player_name),
    FOREIGN KEY (game_id) REFERENCES games(id),
    FOREIGN KEY (player_name, game_id) REFERENCES players(name, game_id),
    FOREIGN KEY (question_id) REFERENCES questions(id)
);

CREATE VIEW game_results AS
SELECT answers.game_id,
    player_name,
    impostor,
    abs(answer_value - true_answer) > abs(answer_value - wrong_answer) as impostor_won
FROM answers
    INNER JOIN game_questions ON answers.question_id = game_questions.question_id AND answers.game_id = game_questions.game_id
    INNER JOIN questions ON game_questions.question_id = questions.id;

-- transform the above to a create view
CREATE VIEW sqlpage_shell AS
SELECT
  'shell' AS component,
  'Corporate Conundrum' AS title,
    'Unleash your inner executive in this thrilling board game of corporate espionage. Make the right choices to lead your company to success!' AS description,
    'affiliate' AS icon,
    '/' AS link,
    '["New Game", "rules"]' AS menu_item,
    'Libre Baskerville' AS font,
    'en-US' AS lang
;