use tokio;

use dnote::server;

pub fn start_server() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let future = server::serve();
    rt.block_on(future);
}
