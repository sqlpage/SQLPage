SELECT 'shell' AS component,
    'SQLPage with a frontend component' AS title,
    'settings' AS icon,

    -- Including react from a CDN like that is quick and easy, but if your project grows larger,
    -- you might want to use a bundler like webpack, and include your javascript file here instead
    'https://cdn.jsdelivr.net/npm/react@18.2.0/umd/react.production.min.js' AS javascript,
    'https://cdn.jsdelivr.net/npm/react-dom@18.2.0/umd/react-dom.production.min.js' AS javascript,
    'my_react_component.js' AS javascript;

SELECT 'react_component' AS component,
        'MyComponent' AS react_component_name,
        'World' AS greeting_name;