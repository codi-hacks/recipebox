# recipebox

Write and host your own recipe website without the need for a database.

This app is an iteration on the original ruby [recipebox](https://github.com/gfax/recipebox) app.

## In progress!!

See the [BLUEPRINTS](https://github.com/codi-hacks/recipebox/issues/1) issue to track development progress.

## Development

It is recommended to use [rustup](https://rustup.rs/) or similar toolchain manager as it will pick up `rust-toolchain.toml` and auto-switch your toolchain to nightly which is needed for certain file io operations.

Start the server:

`cargo run`

The server can be accessed from http://localhost:4000

## Production

Build server binary:

`cargo build`

## Example systemd service file

For production, you may want to control the process via a service manager like systemd. Example:

```
[Unit]
Description=recipebox production server
After=network.target

[Service]
Type=forking
User=www-data
WorkingDirectory=/path/to/recipe/files
ExecStart=/path/to/bin/recipebox

[Install]
WantedBy=multi-user.target
```
