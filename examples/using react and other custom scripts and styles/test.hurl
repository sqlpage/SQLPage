GET http://localhost:8080/
HTTP 200
[Asserts]
body contains "SQLPage with a frontend component"
body contains "Try my react component !"
body contains "style.css"
body contains "funky_text"

GET http://localhost:8080/style.css
HTTP 200
[Asserts]
body contains "#funky_text"
body contains "neon-glow"

GET http://localhost:8080/react.sql
HTTP 200
[Asserts]
body contains "my_react_component.js"
body contains "react_component"
body contains "Dynamic component is loading..."
body contains "World"

GET http://localhost:8080/my_react_component.js
HTTP 200
[Asserts]
body contains "fetch(\"/api.sql\")"
body contains "ReactDOM.createRoot"
