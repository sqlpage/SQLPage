CREATE TABLE blog_posts (
    title TEXT PRIMARY KEY,
    description TEXT NOT NULL,
    icon TEXT NOT NULL,
    external_url TEXT,
    content TEXT,
    created_at TIMESTAMP NOT NULL
);

INSERT INTO blog_posts (title, description, icon, created_at, content)
VALUES
    (
        'SQLPage versus No-Code tools',
        'What are the advantages and disadvantages of SQLPage compared to No-Code tools?',
        'code-minus',
        '2023-08-03',
        '
**No-Code vs Low-Code: Why Writing an App in SQL Makes Sense** 🚀
=================================================================

So, you''ve got this brilliant app idea that''s been keeping you up at night. You want it to shine, sparkle, and dazzle users. But here''s the catch: you''re not exactly a coding wizard. No worries, the tech world has got you covered with two charming suitors – No-Code and Low-Code platforms. 🎩💻

The Tempting Allure of No-Code
------------------------------

**No-Code tools**, oh sweet simplicity! They sweep you off your feet, promising a land of no syntax-induced headaches. You don''t need to be on first-name terms with SQL or worry about the semi-colon''s mood swings. Plus, you get to play the grand designer, arranging elements like a digital Picasso.

But, hold up, there''s a twist in this love story. As the relationship deepens, you discover the truth – No-Code isn''t that great at handling complex data manipulations. Your app''s smooth moves suddenly stumble, and you realize the sleek exterior is covering up some cracks. When the app grows, maintenance turns into a melodrama, and waving goodbye to version control feels like a heartbreak. 💔

The Charming Proposal of Low-Code
---------------------------------

Now enters the **Low-Code** hero, complete with a dapper suit and a trunk full of powerful tools. With Low-Code, you''re in the driver''s seat, crafting every detail of your app with elegance and precision. You''re not just the designer; you''re the maestro orchestrating a symphony of functionality.

But don''t be fooled by the fairy-tale facade – some Low-Code sweethearts have a hidden agenda. They entice you with their ease and beauty, but as your app grows, you discover they''re trying to lock you in. A switch to something more substantial means starting from scratch, leaving you with a déjà vu of rebuilding your app''s entire world.

The SQLPage Love Story 💘
-------------------------

And then, there''s **SQLPage** – the dashing knight that marries the best of both worlds. Lightweight, easy to self-host, and oh-so-elegant, SQLPage dances with your PostgreSQL database, effortlessly creating captivating web apps. It''s like a matchmaking genius, uniting your SQL skills with stunning visual displays. 🕺💃

But here''s the real showstopper – SQLPage doesn''t force you to learn new tricks. It''s all about _standard_ SQL, your old pal from the database kingdom. No code voodoo, no convoluted syntax – just the language you already know and love. And those worries about slow-loading web pages? Say goodbye to buffering frustration; SQLPage websites are sleek, fast, and utterly mesmerizing.

So, next time you''re torn between No-Code''s enchantment and Low-Code''s embrace, remember the charming SQLPage love story. It''s the fairy-tale ending where you''re in control, your data thrives, and your app''s journey grows without painful rewrites. 👑📊

Give your app the love it deserves – the SQLPage kind of love.💕
        '
    );

INSERT INTO blog_posts (title, description, icon, created_at, external_url)
VALUES (
    'Repeating yourself thrice won''t make you a 3X developer',
    'A dive into the traditional 3-tier architecture and the DRY principle, and how tools like SQLPage helps you avoid repeating yourself.',
    'box-multiple-3',
    '2023-08-01',
    'https://yrashk.medium.com/repeating-yourself-thrice-doesnt-turn-you-into-a-3x-developer-a778495229c0'
);