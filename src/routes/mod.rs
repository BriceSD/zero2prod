//! src/routes/mod.rs

pub mod health_check;
pub mod subscriptions;
pub mod subscriptions_confirm;
pub mod newsletter;

pub use health_check::*;
pub use subscriptions::*;
pub use subscriptions_confirm::*;
pub use newsletter::*;
