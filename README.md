# tldr [![build](https://github.com/imandra-ai/tldr/actions/workflows/rust.yml/badge.svg)](https://github.com/imandra-ai/tldr/actions/workflows/rust.yml)

Trace and Log Daemon in Rust

This little daemon runs in the background so that other programs, potentially short lived and spanning multiple unix processes,
can connect to it and send fragments of [TEF](https://docs.google.com/document/d/1CvAClvFfyA5R-PhYUmn5OOQtYMH4h6I0nSsKchNAySU/) traces.
Once the program is done, a `.json` file can be obtained from the daemon.

## Install

```sh
$ cargo install --path .
```

## Use

Run this in the background:
```sh
$ tldr serve
```

When programs have sent traces, they can be listed using:
```sh
$ tldr list
```

Traces are stored as `.jsonl` files (easy to append to, easy to iterate on). Each line is a valid TEF json event.

and you can get a `trace.json` file with:
```
$ tldr get-tef $some_jsonl_path -o trace.json
```

Then this trace.json file can be opened with https://ui.perfetto.dev/ .

## Running the daemon with systemd

A basic unit file is in `data/tldr.service`. It assumes tldr is in the standard path, or was installed
via `cargo install` as above.

Run:
```
$ cp data/tldr.service ~/.config/systemd/user/
$ systemctl daemon-reload --user
$ systemctl enable --user tldr
$ systemctl start --user tldr
```
