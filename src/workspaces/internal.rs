// Internal, standarized, representation of multiple
// Window managers' workspaces.

use std::collections::BTreeSet;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct WorkspaceState {
    active_workspace: i64,
    workspaces: BTreeSet<i64>,
}

pub enum WorkspaceChange {
    Destroy(i64),
    Create(i64),
    Focus(i64),
}

impl Default for WorkspaceState {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceState {
    pub fn new() -> WorkspaceState {
        WorkspaceState {
            active_workspace: 0,
            workspaces: BTreeSet::new(),
        }
    }
    pub fn update(&mut self, optcode: WorkspaceChange) -> bool {
        match optcode {
            WorkspaceChange::Create(id) => self.workspaces.insert(id),
            WorkspaceChange::Destroy(id) => self.workspaces.remove(&id),
            WorkspaceChange::Focus(id) => {
                self.active_workspace = id;
                // Adds the workspaces when they exist before this program starts
                !self.workspaces.insert(id)
            }
        }
    }
}
