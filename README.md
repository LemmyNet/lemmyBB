# lemmyBB
[![Build Status](https://cloud.drone.io/api/badges/LemmyNet/activitypub-federation-rust/status.svg)](https://cloud.drone.io/Nutomic/lemmyBB)

A Lemmy frontend inspired by [phpBB](https://www.phpbb.com/).

## Deployment

Follow these steps to install LemmyBB on your server. Resource usage is very low, so it should work fine with even the smallest of VPS.

First, prepare by cloning the code repository and creating pictrs folder.
```
git clone https://github.com/LemmyNet/lemmyBB.git
cd lemmyBB
mkdir -p docker/volumes/pictrs
chown 991:991 docker/volumes/pictrs
```

Then copy the config, and set your actual hostname. See [this page](https://join-lemmy.org/docs/en/administration/configuration.html) for a full list of configuration options.
```
cp docker/lemmy_config_default.hjson docker/lemmy.hjson
sed -i -e 's/example.com/your-domain.xyz/g' docker/lemmy.hjson
sed -i -e 's/example.com/your-domain.xyz/g' docker/docker-compose.yml
```

Next we start LemmyBB and other services it depends on using docker-compose. This takes relatively long for the first time (about 11 minutes on a 1 cpu vps). Subsequent builds will be faster thanks to caching.

```
apt install docker-compose
docker-compose -f docker/docker-compose.yml up -d
```

Finally we request a TLS certificate from [Let's Encrypt](https://letsencrypt.org/), and configure nginx as reverse proxy. Of course you can set things up differently, but then you need to figure it out yourself.

The nginx config provides access to both LemmyBB (under path `/`), and lemmy-ui (under `/lemmy-ui). The latter is useful because LemmyBB is still missing many features, in particular for moderation. So if you need to remove a post, ban a user or view the mod log, do it via lemmy-ui. Login information is the same as with LemmyBB. You can also decide to remove lemmy-ui, or restrict access with [HTTP Basic Auth](https://docs.nginx.com/nginx/admin-guide/security-controls/configuring-http-basic-authentication/).

```
apt install certbot nginx
# replace with your actual domain and contact email
certbot certonly --nginx --agree-tos -d 'your-domain.xyz' -m 'your-email@abc.com'
# copy nginx config
cp docker/nginx.conf /etc/nginx/sites-enabled/lemmybb.conf
# update nginx config with actual domain
sed -i -e 's/example.com/your-domain.xyz/g' /etc/nginx/sites-enabled/lemmybb.conf
# reload nginx with new config
nginx -s reload
```

Now visit your domain in a browser. If everything went well, you will see a form for creating the initial admin account, and setting the site name.

You should also add the following line to your cron (using `crontab -e`), to automatically refresh the TLS certificate before it expires.

```
@daily certbot certonly --nginx -d 'example.com' --deploy-hook 'nginx -s reload'
```

For more information, you can read the [Lemmy documentation](https://join-lemmy.org/docs/en/index.html), use the [LemmyBB issue tracker](https://github.com/LemmyNet/lemmyBB/issues) or [chat on Matrix](https://matrix.to/#/#lemmy-space:matrix.org).


The instructions above build LemmyBB directly from the local folder. To receive updates with new features and bug fixes, simply pull the git repository and rebuild. You can also easily make modifications to files, or fetch from another git repository with customizations.

```
# update to latest git version
git pull
# optional: manually edit a template file
nano templates/header.html.hbs
# build and deploy from local files
docker-compose -f docker/docker-compose.yml up -d --build
```

## Development

Follow the instructions for [Local Lemmy Development](https://join-lemmy.org/docs/en/contributing/local_development.html). You need the Lemmy backend source code and a cargo installation, along with PostgreSQL. Lemmy-ui is not necessary, but can be useful for testing.

Once the development setup is ready, execute `cargo run` in both the lemmy and lemmyBB directories.

## Configuration

LemmyBB is configured via environment variables:

| var                     | default value         | description                                                  |
|-------------------------|-----------------------|--------------------------------------------------------------|
| LEMMY_BB_BACKEND        | http://localhost:8536 | Protocol, hostname and port where lemmy backend is available |
| LEMMY_BB_LISTEN_ADDRESS | 127.0.0.1:1244        | IP and port where LemmyBB listens for requests               |
## License

The project is licensed under [AGPLv3](LICENSE). 

Theme files from phpBB are licensed under [GPLv2](https://www.phpbb.com/downloads/license).