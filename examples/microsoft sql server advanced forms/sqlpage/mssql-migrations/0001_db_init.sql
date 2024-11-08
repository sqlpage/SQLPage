create table users (
    id int primary key IDENTITY(1,1),
    name varchar(255) not null
);

create table groups (
    id int primary key IDENTITY(1,1),
    name varchar(255) not null
);

create table group_members (
    group_id int not null,
    user_id int not null,
    constraint PK_group_members primary key (group_id, user_id),
    constraint FK_group_members_groups foreign key (group_id) references groups (id),
    constraint FK_group_members_users foreign key (user_id) references users (id)
);

CREATE TABLE questions(
    id INT PRIMARY KEY IDENTITY(1,1),
    question_text TEXT
);

CREATE TABLE survey_answers(
    id INT PRIMARY KEY IDENTITY(1,1),
    question_id INT,
    answer TEXT,
    timestamp DATETIME DEFAULT GETDATE(),
    CONSTRAINT FK_survey_answers_questions FOREIGN KEY (question_id) REFERENCES questions(id)
);

INSERT INTO questions(question_text) VALUES
    ('What is your name?'),
    ('What is your age?'),
    ('What is your favorite color?');
