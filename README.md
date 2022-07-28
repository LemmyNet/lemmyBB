# lemmyBB
[![Build Status](https://cloud.drone.io/api/badges/LemmyNet/activitypub-federation-rust/status.svg)](https://cloud.drone.io/Nutomic/lemmyBB)

A Lemmy frontend inspired by [phpBB](https://www.phpbb.com/).

## Deployment

Follow these steps to install LemmyBB on your server. Resource usage is very low, it should work fine with even the smallest of VPS.

```
git clone https://github.com/Nutomic/lemmyBB.git
mkdir -p docker/volumes/pictrs
chown 991:991 docker/volumes/pictrs
# copy and adjust lemmy config
docker-compose up -d
# copy nginx config
# request letsencrypt cert
# (re)start nginx
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