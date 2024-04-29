# SQLPage CAS Client

This is a demonstration of how to implement a 
[Central Authentication Service (CAS)](https://apereo.github.io/cas/)
client in a SQLPage application.

CAS is a single sign-on protocol that allows users to authenticate once and access multiple applications without having to log in again. It is primarily used in academic institutions and research organizations.

The protocol is based on a ticketing system, where the user logs in once and receives a ticket that can be used to access other applications without having to log in again. The ticket is validated by the CAS server, which then returns the user's information to the application.

This can be implemented in SQLPage with two `.sql` files: 
 - [`login.sql`](login.sql): This just redirects the user to the CAS server's login page.
 - [`redirect_handler.sql`](redirect_handler.sql): This is the page where the CAS server redirects the user after login. It validates the ticket by sending a request to the CAS server and if the ticket is valid, it creates a session for the user in the SQLPage application.

## Configuration

To use this CAS client in your own SQLPage application, you need to follow these steps:

1. Configure your CAS server to allow your SQLPage application to authenticate users. You will need to create a new service in the CAS server with the following information:
   - **Service URL**: The URL of your `redirect_handler.sql` page. For example, `https://example.com/redirect_handler.sql`.
   - **Service Name**: A descriptive name for your service. This can be anything you want.
   - **Service Type**: `CAS 3.0`.
2. In your SQLPage application, set the following environment variable:
    - `CAS_ROOT_URL`: The URL of your CAS server. For example, `https://cas.example.com/cas`.

> Environment variables are global variables that can be made available to a program.
> Using environment variables is a good practice for storing sensitive information and configuration settings,
> so that they are not hard-coded in the code and are easy to change without modifying the code.
> You can set an environment variable by running `export VARIABLE_NAME=value` in the terminal before starting your SQLPage application.
> If you are running your application as a [systemd](https://en.wikipedia.org/wiki/Systemd) service,
> you can set environment variables in the service configuration file, like this:
> ```ini
> [Service]
> Environment="VARIABLE_NAME=value"
> ```
>
> Alternatively, you could store the CAS root URL inside your database and replace
> `sqlpage.environment_variable('CAS_ROOT_URL')` with `(SELECT cas_root_url FROM cas_config)`
> in the `login.sql` and `redirect_handler.sql` files.

## CAS v3 Authentication Flow, step by step

### Login
The client (usually a web browser) requests a resource from the application (client service).
The application redirects the client to the CAS server with a service URL (the URL to which CAS should return the user after authentication).

### CAS Server Authentication
The CAS server presents a login form.
The user submits their credentials (username and password).
Upon successful authentication, the CAS server redirects the user back to the application with a service ticket (ST) appended to the service URL.

### Service Ticket Validation
The application receives the service ticket and makes a back-channel request to the CAS server to validate the service ticket.
The CAS server responds with a success or failure. If successful, it also returns the user's attributes (such as username, email, etc.).

### User Session
Upon successful validation, the application creates a session for the user and grants access to the requested resource.

### CAS v3 Pseudocode Implementation

```plaintext
function authenticateUser(serviceUrl):
    if userNotLoggedIn():
        redirectToCasServer(serviceUrl)

function redirectToCasServer(serviceUrl):
    casLoginUrl = "https://cas.example.com/login?service=" + urlEncode(serviceUrl)
    redirect(casLoginUrl)

function casCallback(request):
    serviceTicket = request.getParameter("ticket")
    if serviceTicket is not None:
        validationUrl = "https://cas.example.com/serviceValidate?ticket=" + serviceTicket + "&service=" + urlEncode(serviceUrl)
        validationResponse = httpRequest(validationUrl)
        if validateResponse(validationResponse):
            userAttributes = extractAttributes(validationResponse)
            createUserSession(userAttributes)
            redirectToService(serviceUrl)
        else:
            authenticationFailed()
    else:
        error("Invalid service ticket.")
```

## Notes

- This implementation uses the CAS 3.0 protocol. If your CAS server uses a different version of the protocol, you may need to modify the code (the ticket validation URL in redirect_handler.sql in particular).
- This implementation does not handle single sign-out (SLO) or proxy tickets. These features can be added by extending the code in `redirect_handler.sql`.
- This implementation assumes that the CAS server returns the user's email address in the `mail` attribute of the user's profile. If your CAS server uses a different attribute to store the email address, or does not return the email address at all, you will need to modify the code to extract the email address from the user's profile in `redirect_handler.sql`.