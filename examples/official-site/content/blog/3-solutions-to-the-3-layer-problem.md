---
title: 3 solutions to the 3 layer problem
date: '2023-08-10'
icon: adjustments-question
summary: What is the 3 layer problem, and how SQLPage solves it?
---

# 3 solutions to the 3 layer problem

> Some interesting questions emerged from the article [Repeating yourself thrice doesn't turn you into a 3X developer](https://yrashk.medium.com/repeating-yourself-thrice-doesnt-turn-you-into-a-3x-developer-a778495229c0).
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
It can follow a Model-View-Controller pattern or not, it doesn't matter.

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
