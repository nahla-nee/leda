pub mod client;
pub mod header;
pub mod response;
pub mod error;
mod util;

pub use client::Client;
pub use header::Header;
pub use response::Response;
pub use error::Error;
pub use util::gemtext_to_html;