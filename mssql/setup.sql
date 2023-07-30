IF DB_ID('sqlpage') IS NULL
    BEGIN
        CREATE DATABASE sqlpage;
    END;
GO

USE sqlpage;
GO

CREATE LOGIN root WITH PASSWORD = 'secret';
CREATE USER root FOR LOGIN MyUser;
GO