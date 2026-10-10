CREATE TABLE stories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    publication_date TEXT NOT NULL,
    contents_md TEXT NOT NULL,
    optional_contents_md TEXT,
    image TEXT,
    website TEXT,
    git_repository TEXT,
    tags JSON
);

INSERT INTO stories(title, publication_date, contents_md, optional_contents_md, image, website, git_repository, tags) VALUES
    (
        'Greater Lincolnshire species data bank',
        '2025-11-20 21:15:00',
        'This is an SQLPage based system I''ve developed for the organisation I work for to collate information on species within our area, running on FreeBSD and MariaDB. Collating the information is still a work in progress, but there''s a complete example data sheet at https://glincsson.glnp.org.uk/view?taxon_list_item_key=NBNORG0000018213 which pulls in all the various elements of information and a complete group listing at https://glincsson.glnp.org.uk/taxon_group?taxon_group=60 showing various colour coding and indicators.',
        'SQLPage has made it trivial to implement this and allowed us to easily add in new aspects based on feedback from others. Most of it is stock SQLPage, but there are a couple of simple modified/custom components, with some custom javascript to allow saving png images of the maps. Its something we''ve long been wanted to do and have attempted in various guises over the years but using SQLPage is the first time we''ve been able to achieve exactly what we wanted (and more). And it''s been fun!',
        'glsdb.jpg',
        'https://glincsson.glnp.org.uk',
        NULL,
        '["MariaDB"]'
    );