IF DB_ID('sqlpage') IS NULL
    BEGIN
        CREATE DATABASE sqlpage;
    END;
GO

USE sqlpage;
GO

CREATE USER root WITH PASSWORD = 'secret';
GO