apnotes(5)

# NAME

apnotes - interact with notes that got written on note applications from apple devices

# DESCRIPTION

The config file is pretty simple, just provide your e-mail address and login data. On linux you have
the ability to choose between storing your password in plain text in the config file, or if you want to
let the tool querey it via the secret-service api.

# KEYS

*username*
    Your login username
*imap_server*
    Address of your mail server
*email*
    Your email address
*password_type*
    PLAIN or SECRET_SERVICE
*secret_service_attribute*
    Key of the object secret service should query
*secret_service_value*
    Value of the object secret service should query
*editor*
    editor that should be used for editing
*editor_arguments*
    arguments that are getting passed to the editor

# EXAMPLES

Secret service config:
```
username=your_username
imap_server=your_imap_server_address.org
email=your_email@server.org
password_type=SECRET_SERVICE
editor_arguments=
editor=gedit
secret_service_attribute=mail
secret_service_value=mailpw
```

# SEE ALSO

*apnotes*(1)