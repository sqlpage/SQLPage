# **Hosting SQLPage Behind a Reverse Proxy**

Hosting SQLPage behind a reverse proxy can help with security, scalability, and flexibility. 
In this guide, we will guide you step-by-step on how to host SQLPage behind a reverse proxy using
[NGINX](https://www.nginx.com/).

## Why host SQLPage behind a Reverse Proxy ?

Here are some reasons why you might want to host SQLPage behind a reverse proxy:

 - **customize your application's URLs**, removing `.sql` extensions and changing URL parameters
 - **protect against attacks** such as denial-of-service (DoS)  by rate limiting incoming requests
 - **improve performance** by caching responses and serving static files without involving SQLPage
 - **enable HTTPS** on the front-end, even when SQLPage is running on HTTP
 - **host multiple applications** or multiple instances of SQLPage on the same server

## Prerequisites

Before you begin, you will need the following:

 - A server running SQLPage. In this guide, we will assume SQLPage is running on `localhost:8080`
 - Nginx installed on your server. On Ubuntu, you can install NGINX using `sudo apt install nginx`
 - A domain name pointing to your server (optional)
 - An SSL certificate obtained from Certbot (optional)

## Configuring the Reverse Proxy

NGINX uses a hierarchical configuration structure. The global configuration file (`/etc/nginx/nginx.conf`) contains settings that apply to the entire server, such as logging, caching, and rate limiting.Site-specific configuration files, stored in `/etc/nginx/sites-available/`, contain directives for individual websites or applications. These site-specific configurations are activated by creating symbolic links in the `/etc/nginx/sites-enabled/` directory. This setup allows for clean and organized management of multiple sites on a single server.

To host SQLPage behind a reverse proxy, you will need to create a new configuration file in the `/etc/nginx/sites-available/` directory, and then create a symbolic link to it in the `/etc/nginx/sites-enabled/` directory.

Create a file named `sqlpage` in the `/etc/nginx/sites-available/` directory:
```bash
sudo nano /etc/nginx/sites-available/sqlpage
```

Add the following configuration to the file:

```nginx
server {
    listen 80;
    server_name example.com;

    location / {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}
```

Save the file and create a symbolic link to it in the `/etc/nginx/sites-enabled/` directory:
```bash
sudo ln -s /etc/nginx/sites-available/sqlpage /etc/nginx/sites-enabled/sqlpage
```

Test the configuration and reload NGINX:
```bash
sudo nginx -t
sudo systemctl reload nginx
```

Your SQLPage instance is now hosted behind a reverse proxy using NGINX. You can access it by visiting `http://example.com`.


### Streaming-friendly proxy settings

SQLPage streams HTML by default so the browser can render results while the database is still sending rows.
If you have slow SQL queries (you shouldn't), you can add the following directive to your location block:

```nginx
proxy_buffering off;
```

That will allow users to start seeing the top of your pages faster,
but will increase the load on your SQLPage server, and reduce the amount of users you can serve concurrently.

Refer to the official documentation for [proxy buffering](https://nginx.org/en/docs/http/ngx_http_proxy_module.html#proxy_buffering), [gzip](https://nginx.org/en/docs/http/ngx_http_gzip_module.html), and [chunked transfer](https://nginx.org/en/docs/http/ngx_http_core_module.html#chunked_transfer_encoding) when tuning these values.

When SQLPage sits behind a reverse proxy, set `compress_responses` to `false` [in `sqlpage.json`](https://github.com/sqlpage/SQLPage/blob/main/configuration.md) so that NGINX compresses once at the edge.

### URL Rewriting

URL rewriting is a powerful feature that allows you to manipulate URLs to make them more readable, search-engine-friendly, and easy to maintain.
In this section, we will cover how to use URL rewriting with SQLPage.

Note that for basic URL rewriting, you can use a simple [`404.sql`](/your-first-sql-website/custom_urls.sql) file to handle custom URLs.
However, for more complex rewriting rules, you can use NGINX's `rewrite` directive.

#### Example: Rewriting `/products/$id` to `/products.sql?id=$id`

Let's say you want your users to access product details using URLs like `/products/123` instead of `/products.sql?id=123`. This can be achieved using the `rewrite` directive in NGINX.

Here is an example configuration:

```nginx
server {
    listen 80;
    server_name example.com;

    location / {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    location /products/ {
        rewrite ^/products/([^/]+)$ /products.sql?id=$1 last;
    }
}
```

This configuration uses the `rewrite` directive to rewrite URLs of the form `/products/$id` to `/products.sql?id=$id`. The `^/products/([^/]+)$` pattern matches URLs that start with `/products/` and captures the dynamic parameter `$id` using parentheses. The `last` flag indicates that this rewrite rule should be the last one to be applied; if the pattern matches, the rewritten URL is passed to the next location block,
in this case, the proxy_pass directive.

**How it Works**

When a request is made to `/products/123`, the rewrite rule is triggered, and the URL is rewritten to `/products.sql?id=123`. The `proxy_pass` directive then forwards the rewritten URL to the SQLPage instance, which processes the request and returns the response.

#### Example: Removing `.sql` Extension from URLs

Let's say you want to remove the `.sql` extension from all URLs to make them cleaner and more user-friendly. This can be achieved using the `rewrite` directive in NGINX.

```nginx
location / {
    
    # When a request doesn't end with a '/' and doesn't have an extension, add '.sql' at the end 
    rewrite ^/((.*/)?[^/.]+)$ /$1.sql last;
    
    proxy_pass      http://localhost:8080;
}
```

### Hosting Multiple Applications

You may want to use the same web server to host SQLPage together with
another application such as a blog, a different website, or another instance of SQLPage.
In this section, we will cover how to host multiple applications behind a reverse proxy using NGINX.

#### Example: Hosting Two Applications with Different domain names

Let's say you want to host two separate instances of SQLPage on the same server, each accessible via a different domain name: `app1.example.com` and `app2.example.com`. This can be achieved by creating two separate configuration files in the `/etc/nginx/sites-available/` directory and then creating symbolic links to them in the `/etc/nginx/sites-enabled/` directory.

Create `/etc/nginx/sites-available/app1`, and `/etc/nginx/sites-available/app2` configuration files,
and add the following configuration to each file, replacing `localhost:8080` and `app1.example.com` with the appropriate values:

```nginx
server {
    listen 80;
    server_name app1.example.com;

    location / {
        proxy_pass http://localhost:8080;
    }
}
```
Then create symbolic links to the configuration files in the `/etc/nginx/sites-enabled/` directory.

#### Hosting on a Subpath

You may have multiple applications to host, but a single domain name to use. In this case, you can host each application on a different subpath of the domain name, for example, `example.com/app1` and `example.com/app2`.

To host SQLPage on a subpath, you can use a single NGINX configuration file with a `location` block that specifies the subpath:

```nginx
server {
    listen 80;
    server_name example.com;

    location /sqlpage {
        proxy_pass http://localhost:8080;
    }
}
```
This configuration sets up a reverse proxy that forwards incoming requests from `example.com/sqlpage` to `localhost:8080`, where SQLPage is running.

And in the SQLPage configuration file, at `./sqlpage/sqlpage.json`,
you can specify the base URL as `/sqlpage`:

```json
{
    "site_prefix": "/sqlpage"
}
```

### IP Rate Limiting

To enable IP rate limiting for your SQLPage instance, you can use the
[`limit_req` module in NGINX](http://nginx.org/en/docs/http/ngx_http_limit_req_module.html).

Define a global rate limiting zone in `/etc/nginx/nginx.conf`:

```nginx
http {
    ...
    limit_req_zone $binary_remote_addr zone=myzone:10m rate=10r/m;
}
```

Then use it in your site's configuration in `/etc/nginx/sites-available/sqlpage`:

```nginx
server {
    ...

    location / {
        limit_req zone=myzone;
        proxy_pass http://localhost:8080;
        ...
    }
}
```
This configuration sets up a reverse proxy that forwards incoming requests from `example.com` to `localhost:8080`, where SQLPage is running, and enables IP rate limiting to prevent abuse.


### Static File Serving

The [`try_files`](https://nginx.org/en/docs/http/ngx_http_core_module.html#try_files) directive in Nginx specifies the files to attempt to serve before falling back to a specified URI or passing the request to a proxy server. It's typically used within a location block to define the behavior when a request matches that location.

```nginx
server {
    listen 80;
    server_name example.com;

    location ~ \.sql$ {
        include sqlpage_proxy.conf;
    }

    location / {
        try_files $uri @reverse_proxy;
    }

    location @reverse_proxy {
        include sqlpage_proxy.conf;
    }
}
```

And in `/etc/nginx/sqlpage_proxy.conf`:

```nginx
proxy_pass http://localhost:8080;
proxy_http_version 1.1;
proxy_set_header Upgrade $http_upgrade;
proxy_set_header Connection 'upgrade';
proxy_set_header Host $host;
proxy_cache_bypass $http_upgrade;
proxy_buffering on;
proxy_set_header X-Real-IP $remote_addr;
proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
proxy_set_header X-Forwarded-Proto $scheme;
proxy_set_header X-Forwarded-Host $host;
```

### Caching and Buffering

To enable caching and buffering for your SQLPage instance, you can use the 
[`proxy_cache`](https://nginx.org/en/docs/http/ngx_http_proxy_module.html#proxy_cache)
and [`proxy_buffering`](https://nginx.org/en/docs/http/ngx_http_proxy_module.html#proxy_buffering)
directives in NGINX.

Declare the cache in `/etc/nginx/nginx.conf` :

```nginx
http {
    ...
    proxy_cache mycache;
    # Cache settings: 1 hour for 200 and 302 responses, 1 minute for 404 responses
    proxy_cache_valid 200 302 1h;
    proxy_cache_valid 404 1m;
}
```

and then in your sqlpage nginx configuration file `/etc/nginx/sites-available/sqlpage` :

```nginx
server {
    listen 80;
    server_name example.com;

    location / {
        proxy_pass http://sqlpage;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_cache mycache;
        # Buffering: when a client is slow to read the response, quickly read the response from SQLPage and store it in a buffer, then send it to the slow client, while SQLPage can continue processing other requests
        proxy_buffering on;
        proxy_buffer_size 128k;
        proxy_buffers 4 256k;
    }
}
```

### **HTTPS and Certbot**

To let nginx handle HTTPS instead of SQLPage, you can obtain an SSL certificate from Certbot and configure nginx to use it.

Install certbot using the following command:
```bash
sudo snap install --classic certbot
```

Obtain an SSL certificate using the following command:
```bash
sudo certbot --nginx -d example.com
```

### Binding to a UNIX socket

Binding SQLPage to a Unix socket can reduce latency and enhance security by bypassing the network stack and restricting access to the socket file. Unix sockets are suitable for communication within the same host, offering lower overhead compared to TCP/IP.

#### SQLPage Configuration

Edit `./sqlpage/sqlpage.json`. Remove the `listen_on` and `port` configuration entries if they are present.

```json
{
    "unix_socket": "/var/run/sqlpage.sock"
}
```

#### NGINX Configuration

In `/etc/nginx/sites-available/sqlpage`:

```nginx
server {
    listen 80;
    server_name example.com;

    location / {
        proxy_pass http://unix:/var/run/sqlpage.sock;
        proxy_set_header Host $host;
    }
}
```

# Example NGINX configuration for SQLPage

You can find a fully working example of an NGINX configuration for SQLPage 
illustrating all the features described in this guide
in the [examples/nginx](https://github.com/sqlpage/SQLPage/tree/main/examples/nginx)
directory of the SQLPage Github repository.
The example uses Docker and docker-compose to run NGINX and SQLPage.