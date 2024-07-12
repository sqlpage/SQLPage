SELECT 'shell' AS component,
        'Tap Tempo' as title,
        '/' as link,
        'en-US' as lang,
        'A tool to measure a tempo in bpm by clicking a button in rythm.' as description,
        'Vollkorn' as font,
        'music' as icon,
        'Proudly powered by [SQLPage](https://sql.ophir.dev)' as footer;

SELECT 'hero' as component,
        'Tap Tempo' as title,
        'Tap Tempo is a tool to **measure a tempo in bpm** by clicking a button in rythm.' as description_md,
        'drums by Nana Yaw Otoo.jpg' as image,
        sqlpage.link('taptempo.sql', json_object('session', random())) as link,
        'Start tapping !' as link_text;

SELECT 'text' as component,
        'About TapTempo' as title,
        '
## Context

This tool is written in the SQL database query language, and uses the [SQLPage](https://sql.ophir.dev) framework to generate the web interface.

It serves as a demo for the framework.

If what you really want is to measure a tempo, not learn about website building and databases,
you should probably use something else, that does not require a web server and a database to run 😉.

## History

There is a large family of implementations of the tap tempo tool.

It originates from a french linux discussion website, [linuxfr.org](https://linuxfr.org/), where it was implemented in C++ by [mzf](https://linuxfr.org/users/mzf).

[See alternative implementations](https://linuxfr.org/wiki/taptempo).

        ' as contents_md;