create table dog_lover_profiles(
    id integer primary key,
    profile_description text not null,
    score integer not null
);

insert into dog_lover_profiles(profile_description, score)
    values ('I love dogs', 100), ('I hate them', 0);

create table answers(
    id integer primary key,
    profile_id integer not null references dog_lover_profiles(id),
    timestamp timestamp not null default current_timestamp
);