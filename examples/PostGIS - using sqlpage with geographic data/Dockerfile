FROM debian:12-slim

WORKDIR /var/www

ADD https://github.com/lovasoa/SQLpage/releases/download/v0.7.2/sqlpage-linux.tgz .

RUN apt-get update && \
    apt-get -y install sqlite3 libsqlite3-mod-spatialite && \
    tar xvzf sqlpage-linux.tgz && \
    rm sqlpage-linux.tgz
    
COPY . .

CMD ["sqlite3"]
