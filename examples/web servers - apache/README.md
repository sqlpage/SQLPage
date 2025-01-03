# SQLPage with Apache Reverse Proxy

This example demonstrates how to run SQLPage behind the popular Apache HTTP Server.
This is particularly useful when you already have a server running Apache (with a PHP application for example)
and you want to add a SQLPage application.

This setup allows you to:
- Host multiple websites/applications on a single server
- Serve static files directly through Apache
- Route specific paths to SQLPage

## How it Works

Apache acts as a reverse proxy, forwarding requests for `/my_website` to the SQLPage
application while serving static content directly. The configuration uses:

- [`mod_proxy`](https://httpd.apache.org/docs/current/mod/mod_proxy.html) and [`mod_proxy_http`](https://httpd.apache.org/docs/current/mod/mod_proxy_http.html) for reverse proxy functionality
- [Virtual hosts](https://httpd.apache.org/docs/current/vhosts/) for domain-based routing
- [`ProxyPass`](https://httpd.apache.org/docs/current/mod/mod_proxy.html#proxypass) directives to forward specific paths

## Docker Setup

The `docker-compose.yml` defines three services:
- `apache`: Serves static content and routes requests
- `sqlpage`: Handles dynamic content generation
- `mysql`: Provides database storage

## Native Apache Setup

To use this with a native Apache installation instead of Docker:

1. Install Apache and required modules:
```bash
sudo apt install apache2
sudo a2enmod proxy proxy_http
```

2. Configuration changes:
- Place the `httpd.conf` content in `/etc/apache2/sites-available/my-site.conf`
- Adjust paths:
  - Change `/var/www` to your static files location
  - Update SQLPage URL to match your actual SQLPage server address (`http://localhost:8080/my_website` if you are running sqlpage locally)
  - Modify log paths to standard Apache locations (`/var/log/apache2/`)

3. SQLPage setup:
- Install SQLPage on your server
- Configure it with the same `site_prefix` in `sqlpage.json`
- Ensure MySQL is accessible from the SQLPage instance

4. Enable the site:
```bash
sudo a2ensite my-site
sudo systemctl reload apache2
```

## Files Overview

- `httpd.conf`: Apache configuration with proxy rules
- `sqlpage_config/sqlpage.json`: SQLPage configuration with URL prefix
- `static/`: Static files served directly by Apache
- `website/`: SQLPage SQL files for dynamic content
