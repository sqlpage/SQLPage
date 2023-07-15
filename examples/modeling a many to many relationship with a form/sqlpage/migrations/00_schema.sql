-- A simple blog with topics and posts
-- Users can create topics and posts. A topic can have many posts and a post can have many topics.

-- The first step is to create the tables in the database. We will use the following SQL queries:
CREATE TABLE topic (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    icon TEXT NOT NULL,
    UNIQUE (name)
);

CREATE TABLE post (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    content TEXT NOT NULL, -- The contents will be stored in markdown
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    main_topic_id INTEGER REFERENCES topic(id)
);

CREATE TABLE topic_post (
    topic_id INTEGER NOT NULL REFERENCES topic(id),
    post_id INTEGER NOT NULL REFERENCES post(id),
    PRIMARY KEY (topic_id, post_id)
) WITHOUT ROWID;

-- A view of the topics with the number of posts and most recent post
CREATE VIEW topic_with_stats AS
SELECT topic.id,
    topic.name,
    topic.icon,
    COALESCE(count(topic_post.post_id), 0) as nb_posts,
    max(post.created_at) as last_post
FROM topic
    LEFT JOIN topic_post ON topic_post.topic_id = topic.id
    LEFT JOIN post ON post.id = topic_post.post_id
GROUP BY topic.id
ORDER BY nb_posts DESC;