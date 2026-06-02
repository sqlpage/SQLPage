GET http://localhost:8080
HTTP 200
[Asserts]
body contains "Welcome to Corporate Conundrum"
body contains "Start a New Game"
body not contains "An error occurred"

GET http://localhost:8080/New%20Game.sql
HTTP 302
[Captures]
game_id: header "Location" regex "id=(-?\\d+)"
[Asserts]
header "Location" matches "^game\\.sql\\?id=-?\\d+$"

GET http://localhost:8080/game.sql?id={{game_id}}&Player=Alice
HTTP 200
[Asserts]
body contains "Players"
body contains "Alice"
body contains "Add a new player"
body not contains "An error occurred"

GET http://localhost:8080/next-question.sql?game_id={{game_id}}&player=Alice
HTTP 302
[Captures]
question_id: header "Location" regex "question%5Fid=(\\d+)"
[Asserts]
header "Location" contains "question.sql"
header "Location" contains "player=Alice"

GET http://localhost:8080/question.sql?game_id={{game_id}}&question_id={{question_id}}&player=Alice
HTTP 200
[Asserts]
body contains "Submit your answer"
body contains "Your answer"
body not contains "An error occurred"

GET http://localhost:8080/wait.sql?game_id={{game_id}}&question_id={{question_id}}&player=Alice&answer=42
HTTP 200
[Asserts]
body contains "Waiting for other players to answer"
body not contains "An error occurred"

GET http://localhost:8080/next-question.sql?game_id={{game_id}}&player=Alice
HTTP 302
[Asserts]
header "Location" contains "game-over.sql"

GET http://localhost:8080/game-over.sql?game_id={{game_id}}
HTTP 200
[Asserts]
body contains "The game is over"
body contains "Scores"
body contains "Alice"
body not contains "An error occurred"
