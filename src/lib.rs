//! # Webwire command-line Interface
//!
//! [![Crates.io](https://img.shields.io/crates/v/webwire-cli)](https://crates.io/crates/webwire-cli)
//! [![GitHub](https://img.shields.io/github/license/webwire/webwire-cli)](https://github.com/webwire/webwire-cli/blob/master/LICENSE)
//! [![GitHub Workflow Status](https://img.shields.io/github/workflow/status/webwire/webwire-cli/Rust)](https://github.com/webwire/webwire-cli/actions)
//!
//! [![Discord Chat](https://img.shields.io/discord/726922033039933472?label=Discord+Chat&color=%23677bc4&logo=discord&logoColor=white&style=for-the-badge)](https://discord.gg/jjD6aWG)
//!
//! ![webwire logo](https://webwire.dev/logo.svg)
//!
//! Webwire is a **contract-first API system** which features an
//! interface description language a network protocol and
//! code generator for both servers and clients.
//!
//! This repository contains the the **command-line interface** used
//! to validate Webwire IDL files and generate code and documentation.
//!
//! To learn more about webwire in general please visit the documentation
//! repository [webwire/webwire-docs](https://github.com/webwire/webwire-docs).
//!
//! # Example
//!
//! The following example assumes a Rust server and a TypeScript client. Webwire
//! is by no means limited to those two but those languages show the potential of
//! webwire best.
//!
//! Given the following IDL file:
//!
//! ```webwire
//! struct HelloRequest {
//!     name: String,
//! }
//!
//! struct HelloResponse {
//!     message: String,
//! }
//!
//! service Hello {
//!     hello: HelloRequest -> HelloResponse
//! }
//! ```
//!
//! The server and client files can be generated using the code generator:
//!
//! ```bash
//! $ webwire gen rust < api/chat.ww > server/src/api.rs
//! $ webwire gen ts < api/chat.ww > client/src/api.ts
//! ```
//!
//! A Rust server implementation for the given code would look like this:
//!
//! ```rust,ignore
//! use std::net::SocketAddr;
//! use std::sync::{Arc};
//!
//! use async_trait::async_trait;
//!
//! use ::api::chat;
//!
//! use ::webwire::server::hyper::MakeHyperService;
//! use ::webwire::server::session::{Auth, AuthError};
//! use ::webwire::{Response, Router, Server, ConsumerError};
//!
//! struct ChatService {
//!     #[allow(dead_code)]
//!     session: Arc<Session>,
//!     server: Arc<Server<Session>>,
//! }
//!
//! #[async_trait]
//! impl chat::Server<Session> for ChatService {
//!     async fn send(&self, message: &chat::Message) -> Response<Result<(), chat::SendError>> {
//!         let client = chat::ClientConsumer(&*self.server);
//!         assert!(matches!(client.on_message(message).await, Err(ConsumerError::Broadcast)));
//!         Ok(Ok(()))
//!     }
//! }
//!
//! #[derive(Default)]
//! struct Session {}
//!
//! struct Sessions {}
//!
//! impl Sessions {
//!     pub fn new() -> Self {
//!         Self {}
//!     }
//! }
//!
//! #[async_trait]
//! impl webwire::SessionHandler<Session> for Sessions {
//!     async fn auth(&self, _auth: Option<Auth>) -> Result<Session, AuthError> {
//!         Ok(Session::default())
//!     }
//!     async fn connect(&self, _session: &Session) {}
//!     async fn disconnect(&self, _session: &Session) {}
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create session handler
//!     let session_handler = Sessions::new();
//!
//!     // Create service router
//!     let router = Arc::new(Router::<Session>::new());
//!
//!     // Create webwire server
//!     let server = Arc::new(webwire::server::Server::new(
//!         session_handler,
//!         router.clone(),
//!     ));
//!
//!     // Register services
//!     router.service(chat::ServerProvider({
//!         let server = server.clone();
//!         move |session| ChatService {
//!             session,
//!             server: server.clone(),
//!         }
//!     }));
//!
//!     // Start hyper service
//!     let addr = SocketAddr::from(([0, 0, 0, 0], 2323));
//!     let make_service = MakeHyperService { server };
//!     let server = hyper::Server::bind(&addr).serve(make_service);
//!
//!     if let Err(e) = server.await {
//!         eprintln!("server error: {}", e);
//!     }
//! }
//! ```
//!
//! A TypeScript client using the generated code would look like that:
//!
//! ```typescript
//! import { Client } from 'webwire'
//! import api from 'api' // this is the generated code
//!
//! let client = new Client('http://localhost:8000/', [
//!     api.chat.ClientProvider({
//!         async on_message(message) {
//!             console.log("Message received:", message)
//!         }
//!     })
//! ])
//!
//! assert(await client.connect())
//!
//! let chat = api.chat.ServerConsumer(client)
//! let response = await chat.message({ text: "Hello world!" })
//!
//! assert(response.Ok === null)
//! ```
//!
//! ## License
//!
//! Licensed under either of
//!
//! - Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0)>
//! - MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT)>
//!
//! at your option.
pub mod codegen;
pub mod common;
pub mod idl;
pub mod schema;
