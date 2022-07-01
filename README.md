# lemmyBB

A Lemmy frontend inspired by [phpBB](https://www.phpbb.com/).

## Usage

You can easily run lemmyBB on your local computer, to browse a remote Lemmy instance. Cargo (Rust compiler) and git need to be installed. Then run the following commands. Replace lemmy.ml with your own instance.

```
git clone https://github.com/Nutomic/lemmyBB.git
LEMMY_INTERNAL_HOST=https://lemmy.ml cargo run
```

You can also run lemmyBB on a server. Please open an issue for more details.

## Development

Follow the instructions for [Local Lemmy Development](https://join-lemmy.org/docs/en/contributing/local_development.html). You need the Lemmy backend source code and a cargo installation, along with PostgreSQL. Lemmy-ui is not necessary, but can be useful for testing.

Once the development setup is ready, execute `cargo run` in both the lemmy and lemmyBB directories.

## License

The project is licensed under [AGPLv3](LICENSE). 

Theme files from phpBB are licensed under [GPLv2](https://www.phpbb.com/downloads/license).