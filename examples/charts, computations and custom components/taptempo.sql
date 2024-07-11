SELECT 'shell' AS component, 'Tap Tempo' as title, 'Vollkorn' as font, 'music' as icon;

-- See https://www.sqlite.org/lang_datefunc.html (unixepoch is not available in the current version of SQLite)
INSERT INTO tap(tapping_session, day) VALUES ($session, julianday('now'));

SELECT 'big_button' as component,
         COALESCE(
            (SELECT bpm || ' bpm' FROM tap_bpm WHERE tapping_session = $session ORDER BY day DESC LIMIT 1),
            'Tap'
          ) AS text,
         sqlpage.link('taptempo.sql', json_object('session', $session)) as link;

SELECT 'chart' as component, 'BPM over time' as title, 'area' as type, 'indigo' as color, 0 AS ymin, 200 AS ymax, 'BPM' as ytitle;
SELECT * FROM (
    SELECT 
        strftime('%H:%M:%f', day) AS x,
        bpm AS y
    FROM tap_bpm
    WHERE tapping_session = $session
    ORDER BY day DESC LIMIT 10
) ORDER BY x ASC;