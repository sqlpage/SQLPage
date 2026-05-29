FROM keycloak/keycloak:26.6.2

ADD  --chown=1000:0 https://github.com/jacekkow/keycloak-protocol-cas/releases/download/26.6.2/keycloak-protocol-cas-26.6.2.jar \
    /opt/keycloak/providers/keycloak-protocol-cas.jar

COPY ./keycloak-configuration.json /opt/keycloak/data/import/realm.json

CMD ["start-dev", "--import-realm", "--http-port", "8181"]
