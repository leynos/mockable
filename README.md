# mockable

This crate provides usefull traits to make easier to mock your code using [`mockall`](https://github.com/asomers/mockall) crate.

## Getting Started

Add this to your `Cargo.toml`:

```toml
[dependencies]
mockable = { version = "2", features = [ ... ] }

[dev-dependencies]
mockable = { version = "2", features = ["mock"] }
```

## Documentation

[Documentation](https://docs.rs/mockable/latest/mockable/)

## Clock

The [`Clock`](https://docs.rs/mockable/latest/mockable/trait.Clock.html) trait provides a way to mock the current time.

**Note:** This trait is only available when the `clock` feature is enabled.

[Example](examples/clock.rs).

## Command Runner

The [`CommandRunner`](https://docs.rs/mockable/latest/mockable/trait.CommandRunner.html) trait provides a way to mock the execution of commands.

**Note:** This trait is only available when the `cmd` feature is enabled.

[Example](examples/cmd.rs).

## Env

The [`Env`](https://docs.rs/mockable/latest/mockable/trait.Env.html) trait provides a way to mock the environment variables.

[Example](examples/env.rs).

## Native environment

`NativeEnv` reads native `OsStr` keys and returns owned `OsString` values and
complete native key/value snapshots. `DefaultNativeEnv` reads the process
without Unicode conversion; `MockNativeEnv` is available with the existing
`mock` feature. Inject a reader into code that needs native paths or complete
enumeration. Existing `Env` methods and their Unicode behavior are unchanged.
Known string-key callers can also recover a non-Unicode value from
`Env::raw`'s `VarError::NotUnicode` without migrating APIs.

[Example](examples/native_env.rs).

The private integration-test child harness in `tests/native_env.rs` belongs
only to these native-reader regressions. It supplies an isolated environment
through `Command`, checks native bytes or Windows wide units without printing
the environment, and bounds its own child process lifetime. Its controller
uses an independent fixed-key lookup so an adapter defect cannot recursively
spawn test children. It is not a general process runner or a consumer adapter.

## HTTP Server

The [`HttpServer`](https://docs.rs/mockable/latest/mockable/trait.HttpServer.html) trait provides a way to mock a HTTP server.

**Note:** This trait is only available when the `http` feature is enabled.

[Example](examples/http.rs).

## Mock

The [`Mock`](https://docs.rs/mockable/latest/mockable/struct.Mock.html) trait provides a way to mock a function.

[Example](examples/mock.rs).

## System

The [`System`](https://docs.rs/mockable/latest/mockable/trait.System.html) trait provides a way to mock the system.

[Example](examples/sys.rs).

## UUID Generator

The [`UuidGenerator`](https://docs.rs/mockable/latest/mockable/trait.UuidGenerator.html) trait provides a way to mock the UUID generator.

**Note:** This trait is only available when the `uuid` feature is enabled.

[Example](examples/uuid.rs).
