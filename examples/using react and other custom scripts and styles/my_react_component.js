// Here we are using React and ReactDOM directly, but this file could be a compiled
// version of a React component written in JSX.

function MyComponent({ greeting_name }) {
    const [count, setCount] = React.useState(0);
    return React.createElement(
        'button',
        {
            onClick: async () => {
                const r = await fetch('/api.sql');
                const { total_clicks } = await r.json();
                setCount(total_clicks)
            },
            className: 'btn btn-primary'
        },
        count == 0
            ? `Hello, ${greeting_name}. Click me !`
            : `You clicked me ${count} times!`
    );
}

for (const container of document.getElementsByClassName('react_component')) {
    const root = ReactDOM.createRoot(container);
    const props = JSON.parse(container.dataset.props);
    root.render(React.createElement(window[props.react_component_name], props, null));
}