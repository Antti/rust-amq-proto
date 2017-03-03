# rust-amqp [![Build Status](https://travis-ci.org/Antti/rust-amq-protocol.svg?branch=master)](https://travis-ci.org/Antti/rust-amq-protocol) [![Crates.io](https://img.shields.io/crates/v/amq-protocol.svg)](https://crates.io/crates/amq-protocol)

AMQ protocol implementation in pure rust.

> Note:
> The project is still in very early stages of development,
> it implements all the protocol parsing, but not all the protocol methods are wrapped/easy to use.
> Expect the API to be changed in the future.

## Development notes:

The methods encoding/decoding code is generated using codegen.rb & amqp-rabbitmq-0.9.1.json spec.

You need to have rustfmt installed to generate protocol.rs
To generate a new spec, run:

```sh
make
```

To build the project and run the testsuite, use cargo:

```sh
cargo build
cargo test
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
