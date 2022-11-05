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

## Use

### Profile directory

The profile directory `/some/profile` should contain:

    config.toml
    templates/index.html.tera
    files/

See `example-profile`. The `secret_key` in `/some/profile/config.toml` should be hex string, which can be obtained as follows:

    openssl rand -hex 32

The uploaded files will be stored in `files/`

### Invocation

    ROCKET_CONFIG=/some/profile/config.toml depot

The profile directory is where the config `.toml` file is located (_i.e._ the `dirname` of `ROCKET_CONFIG`).
Notice that that `.toml` file can have any name, except it __must__ have `.toml` extension.

## Approot

In our example, the approot is `depot`. It should be configured in 3 places:

1. In `ROCKET_CONFIG` `.toml` file

2. In NGINX config, `location /depot`

3. In NGINX config, `proxy_pass http://127.0.0.1:11111/depot;`

(The same word "depot" in three places.)

## Workdir

This is where the uploaded files are stored. It is the `-w` switch in the command line.
