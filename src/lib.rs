//! Parse ADS-B/Mode-S messages. Messages with the following Downlink Formats (DF) are supported:
//!
//! - **DF 5**: Surveillance identity (squawk code)
//! - **DF 17/18**: Automatic Dependent Surveillance - Broadcast (ADS-B)
//!   - **TC 1-4**: Aircraft identification and category
//!   - **TC 9-18**: Airborne position
//!   - **TC 19**: Airborne velocity

pub mod cpr;
mod crc;
mod parser;
mod types;

pub use parser::*;
pub use types::*;

#[macro_use]
extern crate lazy_static;
