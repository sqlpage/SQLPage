FROM lovasoa/sqlpage:latest

COPY sqlpage /etc/sqlpage
COPY src /var/www
ENV DATABASE_URL=sqlite:///tmp/pokemon.db?mode=rwc
