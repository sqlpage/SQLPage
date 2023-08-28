# Using spatialite to build a geographic data application

## Introduction

This is a small application that uses [spatialite](https://www.gaia-gis.it/fossil/libspatialite/index)
to save data associated to greographic coordinates. 

If you are using a postgres database, see [this other example instead](../PostGIS%20-%20using%20sqlpage%20with%20geographic%20data/),
which implements the same application using the `PostGIS` extension.

### Installation

You need to install the `spatialite` extension for SQLite. On Debian, or Ubuntu, you can do it with:

```bash
sudo apt install libsqlite3-mod-spatialite
```

Then you can run this application normally with SQLPage.

Notice the `sqlite_extensions` configuration parameter in [`sqlpage/sqlpage.json`](./sqlpage/sqlpage.json).

## Screenshots

![](./screenshots/code.png)