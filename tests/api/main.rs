//the gist of this technique
//tests executables are built in parallel but linked sequentially
//by having a single crate you skip this cost  as only a single crate is built with all of the tests

mod helpers;
mod health_check;
mod subscriptions;
mod subscriptions_confirm;