//! src/routes/mod.rs

pub mod health_check;
pub mod subscriptions;
pub mod subscriptions_confirm;
pub mod newsletter;
mod home;
mod login;

pub use health_check::*;
pub use subscriptions::*;
pub use subscriptions_confirm::*;
pub use newsletter::*;
pub use home::*;
pub use login::*;
