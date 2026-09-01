IF DB_ID('sqlpage') IS NULL
    BEGIN
        CREATE DATABASE sqlpage;
    END;
GO

USE sqlpage;
GO

IF SUSER_ID('root') IS NULL
    BEGIN
        CREATE LOGIN root WITH PASSWORD = 'Password123!';
    END;
GO

IF USER_ID('root') IS NULL
    BEGIN
        CREATE USER root FOR LOGIN root;
    END;
GO

GRANT CREATE TABLE TO root;
GRANT ALTER, DELETE, INSERT, SELECT, UPDATE ON SCHEMA::dbo TO root;
GO
