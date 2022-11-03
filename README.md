# key-value-store

This is an implementation of a simple key-value store server written in Rust with Tokio. Servers accepts TCP connections on port 5555 and serves client requests. Each request is an ASCII string containing only lowercase letters of the english alphabet:

## Requests

- `STORE$key$value$`,
  - server answers with `DONE$`,
- `LOAD$key$`,
  - server answers with `FOUND$value$` if there is a pair `key-value` in server's memory,
  - otherwise, server answers with `NOTFOUND$`.

## Usage

1. Clone this repository.
2. Go to its directory and execute `cargo run`.

## 2 solutions

1. Solution with keeping data in the server's memory is on the branch `master`.
2. Solution with keeping data on the disk is on the branch `io`.

## Testing

There are two kinds of tests:
 - unit - testing request parsing,
 - system - testing the whole server.

To run unit tests just execute `cargo test`.

To run system tests:
1. Run the server.
2. Execute `cargo test -- --ignored` in another terminal (or `cargo test -- --include-ignored` if you want to run all tests).
