# Taxonia service

A web service to supplement taxonia!

## How to run

This application relies on the following external serices:
- Postgres 16
- Redis

They can be started automatically with Docker Compose:

`docker compose -f docker-compose.dev.yml up -d`

To run the Poem server, make sure Cargo is installed, and run:

`cargo run`

For local development, the `cargo watch -x run` command is recommended. You can install it with `cargo install cargo-watch`.

## To-do

Planned features:
- User authentication (email or OAUTH)
- GET/POST recent test settings + scores
- Fetch popular tests
- User notes about taxa
