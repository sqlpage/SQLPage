create table users (
    id int primary key auto_increment,
    name varchar(255) not null,
    email varchar(255) not null
);

create table `groups` (
    id int primary key auto_increment,
    name varchar(255) not null
);

create table group_members (
    group_id int not null,
    user_id int not null,
    primary key (group_id, user_id),
    foreign key (group_id) references `groups` (id),
    foreign key (user_id) references users (id)
);

INSERT INTO users (id, name, email) VALUES
(1, 'John Smith', 'john@email.com'),
(2, 'Jane Doe', 'jane@email.com'), 
(3, 'Bob Wilson', 'bob@email.com'),
(4, 'Mary Johnson', 'mary@email.com'),
(5, 'James Brown', 'james@email.com'),
(6, 'Sarah Davis', 'sarah@email.com'),
(7, 'Michael Lee', 'michael@email.com'),
(8, 'Lisa Anderson', 'lisa@email.com'),
(9, 'David Miller', 'david@email.com'),
(10, 'Emma Wilson', 'emma@email.com');

INSERT INTO `groups` (id, name) VALUES 
(1, 'Team Alpha');

INSERT INTO group_members (group_id, user_id) VALUES
(1, 1),
(1, 2),
(1, 3),
(1, 4),
(1, 5);