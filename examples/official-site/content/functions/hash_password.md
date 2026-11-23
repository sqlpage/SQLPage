---
icon: spy
since: 0.7.2
category: identity-and-security
summary: Hash a password with Argon2id for storage.
parameters:
  - name: password
    type: TEXT
    required: true
    description: The password to hash.
---

Hashes a password with the Argon2id variant and outputs it in the [PHC string format](https://github.com/P-H-C/phc-string-format/blob/master/phc-sf-spec.md), ready to store in your users table.

Every call generates a brand new cryptographic salt so that two people choosing the same password still end up with different hashes, which defeats rainbow-table attacks and lets you safely reveal only the hash.

Use this function only when creating or resetting a password (for example while inserting a brand new user): it writes the stored value. Later, at login time, the [authentication component](documentation.sql?component=authentication#component) reads the stored hash, hashes the visitor's password with the embedded salt and parameters, and grants access only if they match.

### Example

```sql
SELECT 'form' AS component;
SELECT 'username' AS name;
SELECT 'password' AS name, 'password' AS type;

INSERT INTO users (name, password_hash) VALUES (:username, sqlpage.hash_password(:password));
```

### Try online

You can try the password hashing function [on this page](/examples/hash_password.sql).
