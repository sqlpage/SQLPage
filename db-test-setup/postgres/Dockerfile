FROM postgres:16-alpine

# Copy the SSL certificates

COPY --chown=postgres:postgres ./ssl /ssl
RUN chmod 700 /ssl && chmod 600 /ssl/*

CMD ["postgres", "-c", "ssl=on", "-c", "ssl_cert_file=/ssl/server.crt", "-c", "ssl_key_file=/ssl/server.key"]