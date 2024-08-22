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

## Protocol

Clients communicate with the `tldr` daemon via a unix socket (by default in `/tmp/tldr.socket`).
Each client process should open one connection to `tldr` and send these messages:

| message | comment |
|---|---|
| OPEN <trace-id> |  mandatory first message |
| {"ph": "X", …} | a normal TEF event |
| EMIT_TEF <path/to/trace.json> | optional last message |


All processes in a single program run must open the same `trace_id` (a utf-8 safe identifier
used to name the `.jsonl`  file). Traces from processes using the same `trace_id` will
be written to a single `.jsonl` file and will belong in the same trace.

Events can be sent normally after the first `OPEN`, one json event per line.

At the end, one of the processes can send `EMIT_TEF /foo/trace.json` to have the server
write the whole trace, in TEF format (not `.jsonl`! rather, a single json object)
to the file at `/foo/trace.json`.
