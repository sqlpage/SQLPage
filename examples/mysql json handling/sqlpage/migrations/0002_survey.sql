CREATE TABLE questions(
    id INT PRIMARY KEY AUTO_INCREMENT,
    question_text TEXT
);

CREATE TABLE survey_answers(
    id INT PRIMARY KEY AUTO_INCREMENT,
    question_id INT,
    answer TEXT,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (question_id) REFERENCES questions(id)
);


INSERT INTO questions(question_text) VALUES
    ('What is your name?'),
    ('What is your age?'),
    ('What is your favorite color?');
