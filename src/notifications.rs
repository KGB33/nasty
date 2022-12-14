use std::collections::HashMap;

use tokio;

use crate::dnote::{client, server};

pub fn send_test_note() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let future = client::notify(
        "nasty",
        0,
        "an icon",
        "A Test",
        "This is a test notification from nasty.",
        &[],
        HashMap::new(),
        5000,
    );
    rt.block_on(future);
}

pub fn close_notification(id: u32) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let future = client::close_notification(id);
    rt.block_on(future);
}

pub fn start_server() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let future = server::serve();
    rt.block_on(future);
}
