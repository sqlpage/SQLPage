FROM apereo/cas:7.0.4

RUN \
    keytool -genkey -keyalg RSA -alias cas -keystore /etc/cas/thekeystore -storepass changeit -validity 9999 -keysize 2048 -dname "cn=cas.local, ou=MyOU, o=MyCompany, c=FR, st=Nord, l=MyCity" && \
    keytool -genkey -keyalg RSA -alias cas -keystore $JAVA_HOME/lib/security/cacerts -storepass changeit -validity 9999 -keysize 2048 -dname "cn=cas.local, ou=MyOU, o=MyCompany, c=FR, st=Nord, l=MyCity";
