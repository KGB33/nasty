use std::collections::BTreeMap;
use std::net::Shutdown;
use std::process;
use std::{env, io::BufReader};
use std::io::BufRead;
use std::os::unix::net::UnixStream;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
struct WorkspaceState {
    active_workspace: u8,
    wss: BTreeMap<u8, String>,
    ws_names: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct WorkspaceJson {
    id: u8,
    lastwindowtitle: String
} 

#[derive(Serialize, Deserialize)]
struct InnerActiveWindow {
    id: u8
}

#[derive(Serialize, Deserialize)]
struct ActiveWindow {
    workspace: InnerActiveWindow,
    title: String
} 

impl WorkspaceState {
    fn new() -> WorkspaceState {
        // Generate current workspaces
        let data = process::Command::new("hyprctl")
            .arg("-j")
            .arg("workspaces")
            .output()
            .expect("Couldn't get current ws setup");
        let data: Vec<WorkspaceJson> = serde_json::from_slice(&data.stdout)
            .expect("Err converting to json.");
        let mut map = BTreeMap::new();
        for ws in data {
            map.insert(ws.id, ws.lastwindowtitle);
        }
        // Determine active workspace
        let data = process::Command::new("hyprctl")
            .arg("-j")
            .arg("activewindow")
            .output()
            .expect("Couldn't get current active window.");
        let data: ActiveWindow = match serde_json::from_slice(&data.stdout) {
            Ok(value) => value,
            Err(_) => ActiveWindow { workspace: InnerActiveWindow { id: 1 }, title: String::from("eww loading...") }
        };
        WorkspaceState { active_workspace: data.workspace.id, wss: map.to_owned(), ws_names: map.into_keys().collect()}
    }

    fn update(&mut self, opcode: String, value: String) -> bool{
        match opcode.as_str() {
            "workspace" => {
                self.active_workspace = u8::from_str_radix(&value, 10).expect("err converting to int.");
                return true;
            },
            "createworkspace" => {
                let value = u8::from_str_radix(&value, 10).expect("err converting to int.");
                self.wss.insert(value, String::new());
                self.ws_names = self.wss.to_owned().into_keys().collect();
                return true;
            },
            "destroyworkspace" => {
                let value = u8::from_str_radix(&value, 10).expect("err converting to int.");
                self.wss.remove(&value);
                self.ws_names = self.wss.to_owned().into_keys().collect();
                return true;
            },
            "activewindow" => {
                if value != "," { 
                    let value = value.split(",").collect::<Vec<_>>();
                    self.wss.insert(self.active_workspace, value[1].to_string());
                    return true;
                }
                return false;
            },
            _ => {
                return false;
            },
        }
    }
    
}
pub fn hyperland_wm() {
    let hypr_id = env::var("HYPRLAND_INSTANCE_SIGNATURE").expect("Is Hyperland running?");
    let addr = format!("/tmp/hypr/{hypr_id}/.socket2.sock"); 
    let mut state = WorkspaceState::new();
    let u_stream = UnixStream::connect(addr)
                       .expect("Couldn't connect to the server...");
    let stream = BufReader::new(u_stream.try_clone().expect("Couldn't clone socket"));
    ctrlc::set_handler(move ||{
        u_stream.shutdown(Shutdown::Read).expect("shutdown function failed");
        println!("Closing socket reader.")
    }).expect("Error setting Ctrl-C handler");
    for line in stream.lines() {
        let line = line.expect("");
        let line = line.split(">>").collect::<Vec<_>>();
        if state.update(line[0].to_string(), line[1].to_string()) {
            let out = serde_json::to_string(&state).expect("");
            println!("{}", out);
        }
    }
}
