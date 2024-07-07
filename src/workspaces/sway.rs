use swayipc::{Connection, EventType, Fallible};

fn _listen_and_print() -> Fallible<()> {
    let subs = [EventType::Workspace];
    for event in Connection::new()?.subscribe(subs)? {
        let ws = match event? {
            swayipc::Event::Workspace(ws) => ws,
            _ => continue, // We're only listening for Workspace events
        };
        println!("{:?}", *ws)
    }
    Ok(())
}
pub fn listen_and_print() {
    match _listen_and_print() {
        Ok(_) => return,
        Err(e) => panic!("{}", e),
    }
}
