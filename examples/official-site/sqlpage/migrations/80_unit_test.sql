-- unit_test Component Documentation

-- Component Definition
INSERT INTO component(name, icon, description, introduced_in_version) VALUES
    ('unit_test', 'stethoscope', '
A **unit test** verifies individual components in isolation—for database operations, this means testing queries, stored procedures, or data transformations. For REST APIs, unit tests validate handlers, ensuring proper request parsing, business logic execution, and correct response formatting. 

Unit tests offer several key benefits:

- **Early bug detection**: Catch errors immediately when code changes, reducing costly fixes later.
- **Confidence in refactoring**: Modify code freely knowing tests will flag regressions.
- **Documentation**: Tests serve as living examples of how components should behave.
- **Isolation**: Pinpoint failures to specific units rather than hunting through complex systems.
- **Faster debugging**: Failures occur close to the source, making root cause analysis quicker.
- **Database/API stability**: Ensure data integrity and correct endpoint responses without relying on slow integration runs.

A typical **unit test** follows a three-phase structure:

1. **Setup**: Prepare the test environment by initializing dependencies, creating mock objects, and inserting test data (for databases, this might mean seeding a test dataset or establishing a connection).

2. **Execution**: Run the actual test—the code under test is invoked with predefined inputs, and assertions verify the outputs match expected results (e.g., API returns correct status codes, database queries produce accurate records).

3. **Teardown**: Clean up to leave the environment unchanged for subsequent tests. For databases, common strategies include: executing SQL DELETE statements to remove inserted data, wrapping the entire test in a transaction that rolls back at the end (leaving no trace), or using **savepoints** to revert to a clean state mid-test without closing the connection. This isolation ensures tests remain repeatable and don''t interfere with each other.

Unit tests are typically organized into test units, where each unit groups multiple tests that target the same component, feature, or area of functionality. 

With SQLPage, test units take the form of `.sql` files where each file contains several unit tests validating a given treatment (typically a query or transformation). 

```sql
-- ---------------------
-- setup
-- ---------------------
begin;
insert into artists(name) values(''Kate Bush'');

-- ---------------------
-- unit tests
-- ---------------------
select
    ''unit_test'' as component,
    ''Examples of unit tests'' as test_suite;
select
    ''The artists table exists'' as title,
    (select exists(
        select 1 from sqlite_schema where type=''table'' and name=''artists''
    )) as success;
select
    ''The artists table contains ''Kate Bush'''' AS title,
    exists (
        select 1
        from artists
        where name = ''Kate Bush''
    ) as success,
    ''Search for this artist in the table’s content.'' AS description;

-- ---------------------
-- teardown
-- ---------------------
rollback;
```

The example below illustrates the use of savepoints instead of a conventional transaction.

```sql
-- ---------------------
-- setup
-- ---------------------
SAVEPOINT setup;
insert into artists(name) values(''Kate Bush'');

-- ---------------------
-- unit tests
-- ---------------------
select
    ''unit_test'' as component,
    ''Examples of unit tests'' as test_suite;
select
    ''The artists table exists'' as title,
    (select exists(
        select 1 from sqlite_schema where type=''table'' and name=''artists''
    )) as success;
select
    ''The artists table contains ''Kate Bush'''' AS title,
    exists (
        select 1
        from artists
        where name = ''Kate Bush''
    ) as success,
    ''Search for this artist in the table’s content.'' AS description;

-- ---------------------
-- teardown
-- ---------------------
ROLLBACK TO SAVEPOINT setup;
RELEASE SAVEPOINT setup;
```

Using a transaction or a savepoint is only useful if your test unit needs to modify, create, or delete data in your database.

These test units are stored together in a common directory, providing a centralized structure for the whole test suite. Execution works through an `index.sql` file that references each test unit using several `dynamic` components, running them one after another in sequence. Because the tests always execute in the same deterministic order, results are reproducible, dependencies between units are predictable, and any failure can be traced precisely to its position in the run.

In the example below, two test units are located in a directory named `t`, placed at the root of the project. 

```
project/
├── index.sql
└── t/
    ├── index.sql
    ├── tests_artists_table.sql
    └── tests_api.sql
```

This `index.sql` file, also located in the `t` directory, runs the unit tests contained in the test units.

```sql
SELECT 
    ''dynamic'' AS component,
    sqlpage.run_sql(''t/tests_artists_table.sql'') AS properties;

SELECT 
    ''dynamic'' AS component,
    sqlpage.run_sql(''t/tests_api.sql'') AS properties;
```

You can therefore launch all of your tests by calling the `index.sql` file, or execute each test unit separately.
', '0.47.0');

-- Inserting parameter information for the button component
INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'unit_test', * FROM (VALUES
    -- Top-level parameters (for the whole button list)
    ('test_suite', 'A title for the test suite', 'TEXT', TRUE, TRUE),
    -- Item-level parameters (for each button)
    ('success', 'indicates whether the test passed', 'BOOLEAN', FALSE, FALSE),
    ('title', 'A title for the test', 'TEXT', FALSE, FALSE),
    ('description', 'A description of the test for documentation purposes', 'TEXT', FALSE, TRUE)
) x;

INSERT INTO example(component, description, properties) VALUES
    ('unit_test', 'A simple unit test example that always passes and fails, demonstrating the structure of a unit test result.',
    json('[{"component":"unit_test","test_suite": "Examples of unit tests"}, 
        {
            "success": true, 
            "title": "The test passed successfully.",
            "description": "This is a simple test that always passes."
        },
        {
            "success": false, 
            "title": "The test failed.",
            "description": "This is a simple test that always fails."
        }
    ]')
    );
      