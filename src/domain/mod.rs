mod new_subscriber;
mod subscriber_name;
mod subscriber_email;
//wrapper type(tuple struct) to ensure the variant
//name is not empty
pub use subscriber_name::SubscriberName;
pub use new_subscriber::NewSubscriber;
pub use subscriber_email::SubscriberEmail;

