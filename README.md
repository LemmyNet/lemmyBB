# lemmyBB

[![Build Status](https://cloud.drone.io/api/badges/LemmyNet/lemmyBB/status.svg)](https://cloud.drone.io/LemmyNet/lemmyBB)

A Lemmy frontend based on [phpBB 3.3](https://www.phpbb.com/).

[Support forum](https://fedibb.ml/viewforum?f=3) | [Matrix chat](https://matrix.to/#/#lemmybb:matrix.org)

You can help to translate the project via [Weblate](https://weblate.join-lemmy.org/projects/lemmy/lemmybb/).

## Screenshots

![](./screenshots/lemmybb_1.png)
![](./screenshots/lemmybb_2.png)
![](./screenshots/lemmybb_3.png)

## Instances

Here is a list of known lemmyBB instances:

| Domain                                                             | Registration | lemmy-ui domain                                                | Notes                   |
|--------------------------------------------------------------------|--------------|----------------------------------------------------------------|-------------------------|
| [fedibb.ml](https://fedibb.ml/)                      | open         | | Flagship instance for lemmyBB |
| [lemmybb.rollenspiel.monster](https://lemmybb.rollenspiel.monster) | open         | [lemmy.rollenspiel.monster](https://lemmy.rollenspiel.monster) | topic role play         |

Please open a pull request if you know another instance.

## Installation

### New installation (docker-compose)

Follow these instructions to setup a new Lemmy installation on your server, with both lemmybb (for users) and lemmy-ui (mainly for moderation features which are not supported in lemmybb yet). 

Install dependencies and create folders:
```
apt install docker-compose docker.io nginx certbot python3-certbot-nginx
mkdir /srv/lemmybb
cd /srv/lemmybb
mkdir volumes/pictrs
chown 991:991 volumes/pictrs/
```

Download config files, edit lemmy.hjson with your actual domain and make other changes if desired:
```
wget https://raw.githubusercontent.com/LemmyNet/lemmyBB/main/docker/docker-compose.yml
wget https://raw.githubusercontent.com/LemmyNet/lemmyBB/main/docker/lemmy.hjson
nano lemmy.hjson 
```

Start docker-compose services 
```
docker-compose up -d
```

Request tls certificates (use your actual domains and email)
```
certbot certonly --nginx -d lemmybb.com -m contact@lemmybb.com
certbot certonly --nginx -d lemmyui.com -m contact@lemmyui.com
```

Install nginx config and set correct domains. Note that this config by default doesn't allow direct access to the API nor pictrs. This makes it harder for spam bots, but also means that Lemmy clients cant be used. The nginx config includes instructions for putting lemmy-ui behind HTTP Auth, so that only admins can access it.
```
wget https://raw.githubusercontent.com/LemmyNet/lemmyBB/main/docker/nginx.conf -O /etc/nginx/sites-enabled/lemmybb.conf
sed -i -e 's/$lemmybb_domain/lemmybb.com/g' /etc/nginx/sites-enabled/lemmybb.conf
sed -i -e 's/$lemmyui_domain/lemmyui.com/g' /etc/nginx/sites-enabled/lemmybb.conf
nginx -s reload
```

Add these lines to daily cronjob (sudo crontab -e) to renew tls certificates
```
@daily certbot certonly --nginx -d lemmybb.com --deploy-hook 'nginx -s reload'
@daily certbot certonly --nginx -d lemmyui.com --deploy-hook 'nginx -s reload'
```

### Alongside existing Lemmy instance (native)

Follow the [Lemmy installation instructions](https://join-lemmy.org/docs/en/administration/administration.html) to install Lemmy backend and lemmy-ui first. You will need one (sub)domain for LemmyBB, and another for lemmy-ui.

Then install lemmyBB itself. First, ssh into your server and prepare by cloning the code repository.
```
cd /opt
git clone https://github.com/LemmyNet/lemmyBB.git --recursive
```

Change to the folder and compile Lemmy.
```
cd lemmyBB
LEMMYBB_VERSION=$(git describe --tag --always) cargo build --release
```

Copy the nginx config into the sites-enabled folder and edit it
```
cp docker/nginx.conf /etc/nginx/sites-enabled/lemmybb.conf
```

create systemd service file
```
nano /etc/systemd/system/lemmy_bb.service
```

and insert the following content and adapt 'LEMMYBB_BACKEND' and 'LEMMYBB_LISTEN_ADDRESS' to your installation
```
[Unit]
Description=lemmy_bb
After=network.target

[Service]
Type=simple
WorkingDirectory=/opt/lemmyBB/
Environment="LEMMYBB_BACKEND=http://127.0.0.1:8536"
Environment="LEMMYBB_LISTEN_ADDRESS=127.0.0.1:8703"
Environment="LEMMYBB_INCREASED_RATE_LIMIT=1"
Environment="LD_PRELOAD=libjemalloc.so"
ExecStart=/opt/lemmyBB/target/release/lemmy_bb
Restart=always

[Install]
WantedBy=multi-user.target
```

then activate and start the service and check the status
```
systemctl enable --now lemmy_bb.service
systemctl status lemmy_bb.service
```

Run the following commands to update lemmyBB to the latest development version.
```
cd /opt/lemmyBB
git pull
LEMMYBB_VERSION=$(git describe --tag --always) cargo build --release
systemctl restart lemmy_bb.service
```

## Configuration

### Environment variables

| var                          | default value         | description                                                                                                                |
|------------------------------|-----------------------|----------------------------------------------------------------------------------------------------------------------------|
| LEMMYBB_BACKEND              | http://localhost:8536 | Protocol, hostname and port where lemmy backend is available                                                               |
| LEMMYBB_LISTEN_ADDRESS       | 127.0.0.1:1244        | IP and port where lemmyBB listens for requests                                                                             |
| LEMMYBB_INCREASED_RATE_LIMIT |                       | Set this variable if rate limits of Lemmy backend are increased as in docker/lemmy.hjson. Necessary to render last replies |
| LEMMYBB_VERSION              | unknown version       | Version to be shown in footer. Needs to be set at compile time                                                             |

### Frontpage

Create a file `lemmybb_categories.hjson` with content like the following:
```json
[
  [
    "General"
    [
      "!main@voyager.lemmy.ml"
      "!lemmybb@lemmy.ml"
    ]
  ]
  [
    "Open Source"
    [
      "https://lemmy.ml/c/opensource"
      "https://lemmy.ml/c/linux"
      "https://lemmy.ml/c/rust"
    ]
  ]
]
```
Note, you must subscribe manually to remote communities, so that new activities are federated to your instances.

## Development

First install dependencies and setup the database.
```
sudo apt install git cargo postgresql libssl-dev pkg-config
sudo systemctl start postgresql
sudo -u postgres psql -c "create user lemmy with password 'password' superuser;" -U postgres
sudo -u postgres psql -c 'create database lemmy with owner lemmy;' -U postgres
```

Then start LemmyBB with embedded Lemmy instance.
```bash
git clone https://github.com/LemmyNet/lemmyBB.git --recursive
cd lemmyBB
cargo run --features embed-lemmy
```

Then open http://127.0.0.1:1244 in your browser.

It can also be useful to use a production instance as backend, for example if you notice a bug on a specific instance but don't know what causes it. To do this, run the following command with an instance of your choice.
```
LEMMYBB_BACKEND=https://lemmy.ml cargo run
```

## License

The project is licensed under [AGPLv3](LICENSE).

Theme files from phpBB are licensed under [GPLv2](https://www.phpbb.com/downloads/license).
