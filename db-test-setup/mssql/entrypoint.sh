#!/usr/bin/env bash

/opt/mssql/bin/sqlservr &
pid=$!

# Wait 15 seconds for SQL Server to start up
sleep 15

# Run the setup script to create the DB and the schema in the DB
/opt/mssql-tools18/bin/sqlcmd -S localhost -U sa -P "$SA_PASSWORD" -d master -i setup.sql -No

# Wait for sqlservr to exit
wait -n $pid
