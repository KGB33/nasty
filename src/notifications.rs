use std::collections::HashMap;

use tokio;

use crate::dnote::client::notify;

pub fn send_test_note() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let future = notify(
        "nasty",
        0,
        "an icon",
        "A Test", "This is a test notification from nasty.",
        &[],
        HashMap::new(),
        5000,
        );
    rt.block_on(future);
}
