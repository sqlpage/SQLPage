//! Core HTTP server implementation handling SQL file execution and request processing.
//!
//! For more general information about perfomance in sqlite, read our
//! [performance guide](https://sql-page.com/performance.sql).
//!
//! # Overview
//!
//! The webserver module is responsible for:
//! - Processing incoming HTTP requests
//! - Executing SQL files
//! - Streaming query results to clients
//! - Managing database connections
//! - Handling file uploads and static content
//!
//! # Architecture
//!
//! Key components:
//!
//! - [`database`]: SQL execution engine and query processing
//!   - [`database::execute_queries`]: Streams query results from database
//!   - [`database::migrations`]: Database schema management
//!
//! - [`http`]: HTTP server implementation using actix-web
//!   - Request handling
//!   - Response streaming
//!   - [Content Security Policy](https://sql-page.com/safety.sql) enforcement
//!
//! - [`response_writer`]: Streaming response generation
//! - [`static_content`]: Static asset handling (JS, CSS, icons)
//!

pub mod content_security_policy;
pub mod database;
pub mod error_with_status;
pub mod http;
pub mod http_client;
pub mod http_request_info;
mod https;
pub mod request_variables;

pub use database::Database;
pub use error_with_status::ErrorWithStatus;

pub use database::make_placeholder;
pub use database::migrations::apply;
pub mod oidc;
pub mod response_writer;
pub mod routing;
mod static_content;
