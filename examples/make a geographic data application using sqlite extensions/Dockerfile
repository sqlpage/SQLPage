FROM debian:stable-slim

COPY --from=lovasoa/sqlpage:main /usr/local/bin/sqlpage /usr/local/bin/sqlpage

RUN apt-get update && \
    apt-get -y install libsqlite3-mod-spatialite
    
COPY . .

CMD ["sqlpage"]
