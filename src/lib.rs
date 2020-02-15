//! Parse ADS-B/Mode-S messages. Messages with the following Downlink Formats (DF) are supported:
//!
//! - **DF 17**: Automatic Dependent Surveillance - Broadcast (ADS-B)
//!   - **TC 1-4**: Aircraft identification and category
//!   - **TC 9-18**: Airborne position
//!   - **TC 19**: Airborne velocity

pub mod cpr;
mod parser;
mod types;

pub use parser::*;
pub use types::*;
