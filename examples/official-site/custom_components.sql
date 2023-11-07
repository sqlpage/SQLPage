select 'dynamic' as component, properties FROM example WHERE component = 'shell' LIMIT 1;

select 'text' as component, '

# Creating your own SQLPage components


If you have some frontend development experience, you can create your own components, by placing
[`.handlebars`](https://handlebarsjs.com/guide/) files in a folder called `sqlpage/templates` at the root of your server.

## Web page structure

### The [`shell`](./documentation.sql?component=shell#component) component

Each page in SQLPage is composed of a `shell` component,
which contains the page title and the navigation bar,
and a series of normal components that display the data.

The `shell` component is always present. If you don''t call it explicitly,
it will be invoked with the default parameters automatically before your first component
invocation that tries to render data on the page.

There can be only one `shell` component per site, but you can customize its appearance as you see fit.

## Component template syntax

Components are written in [handlebars](https://handlebarsjs.com/guide/),
which is a simple templating language that allows you to insert data in your HTML.

Here is a simple example of a component that displays a list of items:

```handlebars
<h1>{{title}}</h1>

<ul>
{{#each_row}}
    <li>{{my_property}} {{other_property}}</li>
{{/each_row}}
</ul>
```

If you save this file as `sqlpage/templates/my_list.handlebars`, you can use it in your SQL queries
by calling the `my_list` component:

```sql
SELECT ''my_list'' AS component, ''My list'' AS title;
SELECT first_name AS my_property, last_name AS other_property FROM clients;
```

### Styling

SQLPage uses [tabler](https://tabler.io/) for its default styling.
You can include any of the tabler classes in your components to style them.
Since tabler inherits from [bootstrap](https://getbootstrap.com/), you can also use bootstrap classes.

For instance, you can easily create a multi-column layout with the following code:

```handlebars
<div class="row">
{{#each_row}}
    <div class="col">
        {{my_property}}
    </div>
{{/each_row}}
</div>
```

For custom styling, you can write your own CSS files 
and include them in your page header.
You can use the `css` parameter of the default [`shell`](./documentation.sql?component=shell#component) component,
or create your own custom `shell` component with a `<link>` tag.

### Helpers

Handlebars has a concept of [helpers](https://handlebarsjs.com/guide/expressions.html#helpers),
which are functions that you can call from your templates to perform some operations.

Handlebars comes with [a few built-in helpers](https://handlebarsjs.com/guide/builtin-helpers.html),
and SQLPage adds a few more:

- `eq`, `ne`: compares two values for equality (equal, not equal)
- `gt`, `gte`, `lt`, `lte`: compares two values (greater than, greater than or equal, less than, less than or equal)
- `or`, `and`: combines two boolean values (logical operators)
- `not`: negates a boolean value (logical operator)
- `len`: returns the length of a list or string, or the number of keys in an object
- `stringify`: converts a value to its json string representation, useful to pass parameters from the database to javascript functions
- `parse_json`: parses a json string into a value, useful to accept complex parameters from databases that don''t have a native json type
- `default`: returns the first argument if it is not null, otherwise returns the second argument. For instance: `{{default my_value ''default value''}}`.
- `entries`: returns the entries of an object as a list of `{key, value}` objects.
- `delay` and `flush_delayed`: temporarily saves a value to memory, and outputs it later. For instance:
    - ```handlebars
        {{#if complex_condition}}
            <a href="{{link}}">
            {{#delay}}
            </a>
            {{/delay}}
        {{/if}}
        ...
        {{flush_delayed}}
        ```
- `sort`: sorts a list of values
- `plus`, `minus`, `sum`: mathematical operators
- `starts_with`: returns true if a string starts with another string
- `to_array`: useful to accept parameters that can optionally be repeated:
   - if the argument is a list, returns it unchanged,
   - if the argument is a string containing a valid json list, returns the parsed list,
   - otherwise returns a list containing only the argument
- `array_contains`: returns true if a list contains a value
- `icon_img`: generate an svg icon from a *tabler* icon name
- `markdown`: renders markdown text
- `each_row`: iterates over the rows of a query result
- `typeof`: returns the type of a value (`string`, `number`, `boolean`, `object`, `array`, `null`)

## Overwriting the default components

You can overwrite the default components, including the `shell` component,
 by creating a file with the same name in the `sqlpage/templates` folder.

For example, if you want to change the appearance of the `shell` component,
you can create a file called `sqlpage/templates/shell.handlebars` and write your own HTML in it.
If you don''t want to start from scratch, you can copy the default `shell` component
[from the SQLPage source code](https://github.com/lovasoa/SQLpage/blob/main/sqlpage/templates/shell.handlebars).

## Examples

All the default components are written in handlebars, and you can read their source code to learn how to write your own.
[See the default components source code](https://github.com/lovasoa/SQLpage/blob/main/sqlpage/templates).

Some interesting examples are:

 - [The `shell` component](https://github.com/lovasoa/SQLpage/blob/main/sqlpage/templates/shell.handlebars)
 - [The `card` component](https://github.com/lovasoa/SQLpage/blob/main/sqlpage/templates/card.handlebars): simple yet complete example of a component that displays a list of items.
 - [The `table` component](https://github.com/lovasoa/SQLpage/blob/main/sqlpage/templates/table.handlebars): more complex example of a component that uses 
    - the `eq`, `or`, and `sort` handlebars helpers,
    - the `../` syntax to access the parent context,
    - and the `@key` to work with objects whose keys are not known in advance.

' as contents_md;