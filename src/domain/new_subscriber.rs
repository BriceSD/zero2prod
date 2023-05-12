use crate::domain::{SubscriberName, SubscriberEmail};

#[derive(Debug)]
pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}
