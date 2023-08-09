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

INSERT INTO blog_posts (title, description, icon, created_at, content)
VALUES (
        '3 solutions to the 3 layer problem',
        'What is the 3 layer problem, and how SQLPage solves it?',
        'adjustments-question',
        '2023-08-10',
        '
# 3 solutions to the 3 layer problem

> Some interesting questions emerged from the article [Repeating yourself thrice doesn''t turn you into a 3X developer](https://yrashk.medium.com/repeating-yourself-thrice-doesnt-turn-you-into-a-3x-developer-a778495229c0). 
> This short follow-up article aims to answer them and clarify some points.

Hello all,

I am Ophir Lojkine, the main contributor of the open-source application server **SQLPage**.

The previous article focused on the conventional model of splitting applications into three distinct tiers:

1. a graphical interface (_front-end_),
2. an application server (_back-end_),
3. and a database.

In many projects, this results in three distinct implementations of the application’s data model:

1. First, in SQL, in the form of tables, views, and relationships in the database,
2. Then, in _server side_ languages such as Java, Python, or PHP, to create an API managing access to the data, and to implement the business logic of the application,
3. Finally, in JavaScript or TypeScript to implement data manipulation in the user interface.

![Traditional tiers model](blog/three-layers.svg)

---

The topic of interest here is the duplication of the data model between the different layers,
and the communication overhead between them.
We are not talking about how the code is structured within each layer.
It can follow a Model-View-Controller pattern or not, it doesn''t matter.

This three-layer model has several advantages: 
specialization of the programmers, 
parallelization of work,
scalability,
separation of concerns,
and an optimal exploitation of the capacities of the infrastructure on which each layer is deployed:
web browser, server application and database.

Nevertheless, in large-scale projects,
there is often a certain redundancy of the code between the different layers,
as well as a non-negligible share of code dedicated to communication between them.
For small teams and solo developers, this becomes a major drawback.

## 3 solutions

Fortunately, there are several approaches to solving this problem:

1. For **UI-centric applications** without complicated data processing needs,
you can almost completely abandon server-side development and **directly expose the data to the frontend**.
Open-source tools available in this space include Supabase, PocketBase or Hasura.
2. For **applications with a predominant business logic**, traditional _web frameworks_
solve this problem by centralizing frontend and database control in the backend code.
A common solution involves using an ORM and templating system instead of a dedicated javascript application.
Popular solutions include Django, Ruby on Rails, or Symphony.
3. For simpler applications, it is possible to **avoid both backend and frontend development**
by adopting a _database-first_ approach.
This alternative, although less widespread, allows taking advantage of under-exploited modern capacities of relational databases.
The purpose of the original article was to introduce this lesser known approach.
    * **SQLPage** is representative of this last category,
    which allows designing a complete web application _in SQL_.
    This leads to a loss of control over the precise visual appearance of the application,
    which will get a “standardized” look and feel.
    On the other hand, this translates into significant gains in terms of development speed,
    simplicity and performance.
    This solution is not intended to compete with traditional _frameworks_,
    but rather integrate earlier in the life cycle of a project.
    It thus makes it possible to quickly develop a data structure adapted to the application,
    and to iterate over it while benefiting from continuous visual feedback on the final result.
    Then, when the application grows, it’s easy to add a classic frontend and backend on top of the existing database,
    without having to start from scratch.

Whichever approach is chosen in the end,
a solid understanding of the conventional three-tier architecture,
as well as a clear perspective on the challenges it creates and the possible solutions,
facilitates decision-making and the evolution of the project with the best suited technologies.
        '
    );