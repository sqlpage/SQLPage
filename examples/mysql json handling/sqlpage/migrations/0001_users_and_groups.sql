create table users (
    id int primary key auto_increment,
    name varchar(255) not null
);

create table groups (
    id int primary key auto_increment,
    name varchar(255) not null
);

create table group_members (
    group_id int not null,
    user_id int not null,
    primary key (group_id, user_id),
    foreign key (group_id) references groups (id),
    foreign key (user_id) references users (id)
);