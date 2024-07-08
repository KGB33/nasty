use swayipc::{Connection, EventType, Fallible};

use crate::workspaces::internal::{WorkspaceChange, WorkspaceState};

fn _listen_and_print() -> Fallible<()> {
    let subs = [EventType::Workspace];
    let mut state = WorkspaceState::new();
    for event in Connection::new()?.subscribe(subs)? {
        let swayipc::Event::Workspace(ws) = event? else {
            continue;
        };
        let Some(node) = ws.current else {
            eprintln!("Got a {:?} event with no workspace...?", ws.change);
            continue;
        };
        let id = node
            .name
            .map_or(node.id, |v| v.as_str().parse::<i64>().unwrap_or(node.id));
        match ws.change {
            swayipc::WorkspaceChange::Init => {
                state.update(WorkspaceChange::Create(id));
            }
            swayipc::WorkspaceChange::Empty => {
                state.update(WorkspaceChange::Destroy(id));
            }
            swayipc::WorkspaceChange::Focus => {
                state.update(WorkspaceChange::Focus(id));
            }
            // swayipc::WorkspaceChange::Urgent => todo!(),
            _ => continue,
        }
        if let Ok(data) = serde_json::to_string(&state) {
            println!("{}", data);
        }
    }
    Ok(())
}

pub fn listen_and_print() {
    match _listen_and_print() {
        Ok(_) => (),
        Err(e) => panic!("{}", e),
    }
}
