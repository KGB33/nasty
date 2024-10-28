use std::io::BufRead;
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::{env, io::BufReader};

use super::internal::{WorkspaceChange, WorkspaceState};

fn _listen_and_print(stream: BufReader<UnixStream>) {
    let mut state = WorkspaceState::new();
    for line in stream.lines() {
        let Ok(msg) = line else {
            continue;
        };
        let content = msg.split(">>").collect::<Vec<_>>();
        let [opcode, id] = content[..] else {
            continue;
        };
        let id = id.parse::<i64>().unwrap_or(0);
        match opcode {
            "workspace" => {
                state.update(WorkspaceChange::Focus(id));
            }
            "createworkspace" => {
                state.update(WorkspaceChange::Create(id));
            }
            "destroyworkspace" => {
                state.update(WorkspaceChange::Destroy(id));
            }
            _ => continue,
        }
        if let Ok(data) = serde_json::to_string(&state) {
            println!("{}", data);
        }
    }
}
pub fn listen_and_print() {
    let hypr_id = env::var("HYPRLAND_INSTANCE_SIGNATURE").expect("Is Hyprland running?");
    let runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| {
        let uid = env::var("UID").unwrap_or("1000".into());
        format!("/run/user/{uid}")
    });
    let addr = format!("{runtime_dir}/hypr/{hypr_id}/.socket2.sock");
    let u_stream = UnixStream::connect(addr).expect("Couldn't connect to the server...");
    let stream = BufReader::new(u_stream.try_clone().expect("Couldn't clone socket"));
    ctrlc::set_handler(move || {
        u_stream
            .shutdown(Shutdown::Read)
            .expect("shutdown function failed");
        println!("Closing socket reader.")
    })
    .expect("Error setting Ctrl-C handler");
    _listen_and_print(stream);
}
