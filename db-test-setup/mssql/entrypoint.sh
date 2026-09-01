#!/usr/bin/env bash
set -eu

/opt/mssql/bin/sqlservr &
pid=$!

# Creating the login below fails silently and permanently if it runs before the
# server accepts connections, so wait for it rather than sleeping a fixed delay.
for _ in $(seq 60); do
    if /opt/mssql-tools18/bin/sqlcmd -S localhost -U sa -P "$SA_PASSWORD" -Q "SELECT 1" -b -o /dev/null -No; then
        break
    fi
    sleep 1
done

/opt/mssql-tools18/bin/sqlcmd -S localhost -U sa -P "$SA_PASSWORD" -d master -i setup.sql -b -No

wait -n $pid
