GET http://localhost:8080/
HTTP 200
[Captures]
start_url: xpath "string(//a[normalize-space()='Start tapping !']/@href)"
[Asserts]
body contains "Tap Tempo is a tool to"
body contains "measure a tempo in bpm"
body contains "About TapTempo"

GET http://localhost:8080/{{start_url}}
HTTP 200
[Captures]
tap_url: xpath "string(//a[normalize-space()='Tap']/@href)"
[Asserts]
body contains "Tap"
body contains "BPM over time"

GET http://localhost:8080/{{tap_url}}
HTTP 200
[Asserts]
body contains " bpm"
body contains "BPM over time"
