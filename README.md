# Webwire Command-line Interface
[![Crates.io](https://img.shields.io/crates/v/webwire-cli)](https://crates.io/crates/webwire-cli)
[![GitHub](https://img.shields.io/github/license/webwire/webwire-cli)](https://github.com/webwire/webwire-cli/blob/master/LICENSE)
[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/webwire/webwire-cli/Rust)](https://github.com/webwire/webwire-cli/actions)

[![Discord Chat](https://img.shields.io/discord/726922033039933472?label=Discord+Chat&color=%23677bc4&logo=discord&logoColor=white&style=for-the-badge)](https://discord.gg/jjD6aWG)

![webwire logo](https://webwire.dev/logo.svg)

Webwire is a **contract-first API system** which features an
interface description language a network protocol and
code generator for both servers and clients.

This repository contains the the **command-line interface** used
to validate Webwire IDL files and generate code and documentation.

To learn more about webwire in general please visit the documentation
repository [webwire/webwire-docs](https://github.com/webwire/webwire-docs).

# WORK IN PROGRESS

> **Webwire is not ready to use! All the documentation and code in this
repository is either incomplete or non functional. Don't expect anything
to work, yet. Right now this repository is just a collection of ideas and
preliminary implementations.**

# Example

The following example assumes a Rust server and a TypeScript client. Webwire
is by no means limited to those two but those languages show the potential of
webwire best.

Given the following IDL file:

```webwire
webwire 1.0;

struct HelloRequest {
    name: String,
}

struct HelloResponse {
    message: String,
}

service Hello {
    hello: HelloRequest -> HelloResponse
}
```

The server and client files can be generated using the code generator:

```bash
$ webwire gen rust server api/hello.ww server/src/api.rs
$ webwire gen ts client api/hello.ww client/src/api.ts
```

A Rust server implementation for the given code would look like this:

```rust
use std::net::SocketAddr;
use webwire::{Context, Request, Response}
use webwire::hyper::Server;

mod api;
use api::v1::{Hello, HelloRequest, HelloResponse}; // this is the generated code

struct HelloService {}

impl Hello for HelloService {
    fn hello(&self, ctx: &Context, request: &HelloRequest) -> HelloResponse {
        HelloResponse {
            message: format!("Hello {}!", request.name)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    let service = HelloService {};
    let server = webwire::Server::bind(addr).serve(service);
    server.await
}
```

A TypeScript client using the generated code would look like that:

```typescript
import { Client } from 'api/v1' // this is the generated code

client = new Client('http://localhost:8000/')
const response = await client.hello({ name: 'World' })
assert(response.message === 'Hello World!')
```
