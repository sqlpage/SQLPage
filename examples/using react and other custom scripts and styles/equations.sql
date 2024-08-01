select 'shell' as component,
    'Equations' as title,
    'style.css' as css,
    'settings' as icon,
    'https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js' as javascript;

select 'text' as component, '
Newton''s laws of motion are three physical laws that describe the relationship between the forces \( \overrightarrow{F} \) acting on a body,
the resulting motion \( \overrightarrow{a} \) of the body, and the body''s mass \( m \). 
' as contents;
select 
    'card' as component,
    3      as columns;
select 
    'Inertia' as title,
    'The natural behavior of a body is to move in a straight line at constant speed \( \overrightarrow{v} \) unless acted upon by a force \( \overrightarrow{F} \).' as description,
    TRUE                  as active,
    'arrow-right'       as icon;
select 
    'Force' as title,
    'The acceleration \( \overrightarrow{a} \) of a body is directly proportional to the net force \( \overrightarrow{F_{\text{net}}} \) acting on the it, and inversely proportional to its mass \( m \):
\( \overrightarrow{F_{\text{net}}} = m \overrightarrow{a} \), or
\( \sum \overrightarrow F  = m \frac{\mathrm d \overrightarrow v }{\mathrm d t} \).' as description,
    'rocket'       as icon,
    'red'          as color;
select
    'Action and reaction' as title,
    'For every action, there is an equal and opposite reaction.
If body A exerts a force \( \overrightarrow{F_{\text{A on B}}} \) on body B,
then body B exerts a force \( \overrightarrow{F_{\text{B on A}}} = -\overrightarrow{F_{\text{A on B}}} \) on body A.' as description,
    'arrows-exchange' as icon,
    'orange'         as color;