# depot

## Installation


    cargo install --path .

## NGINX config

    location /depot {
        auth_basic "closed site";
        auth_basic_user_file htpasswd_depot;
        client_max_body_size 100M;
        proxy_pass http://127.0.0.1:11111/depot;
    }

### Password file

    aptitude install apache2-utils

Then, to create the new password file:

    htpasswd -c  htpasswd_depot  username

(this will ask to enter new password for `username`)

To add new `user:password` :

    htpasswd  htpasswd_depot  newusername

### Invocation

    depot -a depot -p 11111 -w ~/depot

## Approot

In this example, the approot is `depot`. It should be configured in 3 places:

1. Command line invocation, switch `-a depot`

2. In NGINX config, `location /depot`

3. In NGINX config, `proxy_pass http://127.0.0.1:11111/depot;`

## Workdir

This is where the uploaded files are stored. It is the `-w` switch in the command line.
