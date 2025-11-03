# SQLPage with NGINX Example

This example demonstrates how to set up SQLPage behind an NGINX reverse proxy using Docker Compose. It showcases various features such as rate limiting, URL rewriting, caching, and more.

## Overview

The setup consists of three main components:

1. SQLPage: The main application server
2. NGINX: The reverse proxy
3. MySQL: The database

## Getting Started

1. Clone the repository and navigate to the `examples/nginx` directory.

2. Start the services using Docker Compose:

   ```bash
   docker compose up
   ```

3. Access the application at `http://localhost`.

## Docker Compose Configuration

The `docker-compose.yml` file defines the services.

### SQLPage Service

The SQLPage service uses the latest SQLPage development image, sets up necessary volume mounts for configuration (on `/etc/sqlpage`) and website (on `/var/www`) files, and establishes a connection to the MySQL database.
It reads http requests from a Unix socket (instead of a TCP socket) for communication with NGINX. This removes the overhead of TCP/IP when nginx and sqlpage are running on the same machine.

### NGINX Service

The NGINX service uses the official Alpine-based image. It exposes port 80 and mounts the SQLPage socket and the [custom NGINX configuration file](nginx/nginx.conf).

### MySQL Service

This service sets up a MySQL database with predefined credentials and a persistent volume for data storage.

## NGINX Configuration

The `nginx.conf` file contains the NGINX configuration:

### Streaming and compression

SQLPage streams HTML as it is generated, so browsers can start rendering before the database finishes returning rows. To keep that behaviour through NGINX, prefer minimal buffering and let the proxy handle compression:

```
    proxy_buffering off;

    gzip on;
    gzip_buffers 2 4k;
    gzip_types text/html text/plain text/css application/javascript application/json;

    chunked_transfer_encoding on;
```

Disabling buffering lowers latency but increases the number of active connections; tune the gzip settings to balance CPU cost versus bandwidth, and re-enable buffering only if you need request coalescing or traffic smoothing. See the [proxy buffering](https://nginx.org/en/docs/http/ngx_http_proxy_module.html#proxy_buffering), [gzip](https://nginx.org/en/docs/http/ngx_http_gzip_module.html), and [chunked transfer](https://nginx.org/en/docs/http/ngx_http_core_module.html#chunked_transfer_encoding) directives for more guidance.

When SQLPage runs behind a reverse proxy, set `compress_responses` to `false` in its configuration (documented [here](https://github.com/sqlpage/SQLPage/blob/main/configuration.md)) so that NGINX can perform compression once at the edge.

### Rate Limiting


```nginx
    limit_req_zone $binary_remote_addr zone=one:10m rate=1r/s;
```


This line defines a rate limiting zone that allows 1 request per second per IP address.

### Server Block


```nginx
    server {
        listen 80;
        server_name localhost;

        location / {
            limit_req zone=one burst=5;

            proxy_pass http://unix:/tmp/sqlpage/sqlpage.sock;
        }
    }
```


The server block defines how NGINX handles incoming requests.


#### URL rewriting:

   
```nginx
            rewrite ^/post/([0-9]+)$ /post.sql?id=$1 last;
```


This line rewrites URLs like `/post/123` to `/post.sql?id=123`.

#### Proxy configuration:

   
```nginx
proxy_pass http://unix:/tmp/sqlpage/sqlpage.sock;
```


   These lines configure NGINX to proxy requests to the SQLPage Unix socket.

#### Caching:

   
```nginx
            # Enable caching
            proxy_cache_valid 200 60m;
            proxy_cache_valid 404 10m;
            proxy_cache_use_stale error timeout http_500 http_502 http_503 http_504;
```


   These lines enable caching of responses from SQLPage.

#### Buffering:

   
```nginx
            # Enable buffering
            proxy_buffering on;
            proxy_buffer_size 128k;
            proxy_buffers 4 256k;
            proxy_busy_buffers_size 256k;
```


   These lines configure response buffering for improved performance.

#### SQLPage Configuration

The SQLPage configuration is stored in `sqlpage_config/sqlpage.json`:


```json
{
  "max_database_pool_connections": 10,
  "database_connection_idle_timeout_seconds": 1800,
  "max_uploaded_file_size": 10485760,
  "compress_responses": false,
  "environment": "production"
}
```


This configuration sets various SQLPage options, including the maximum number of database connections and the environment.

## Application Structure

The application consists of several SQL files in the `website` directory:

1. `index.sql`: Displays a list of blog posts
2. `post.sql`: Shows details of a specific post and its comments
3. `add_comment.sql`: Handles adding new comments

The database schema and initial data are defined in [`sqlpage_config/migrations/000_init.sql`](sqlpage_config/migrations/000_init.sql).