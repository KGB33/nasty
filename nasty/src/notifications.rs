use dnote;

use futures_lite::future;

pub fn close_notification(id: u32) {
    future::block_on(async {
        dnote::client::close_notification(id).await;
    })
}

pub fn listen_forever() {}
