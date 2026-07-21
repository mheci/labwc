//! Virtual workspace management — multiple named workspaces per output.

#[derive(Debug, Clone)]
pub struct Workspace {
    pub name: String,
    pub output_name: Option<String>,
}

impl Workspace {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            output_name: None,
        }
    }
}

pub struct WorkspaceManager {
    pub workspaces: Vec<Workspace>,
    pub current_idx: usize,
    pub last_idx: usize,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        let ws = vec![
            Workspace::new("1"),
            Workspace::new("2"),
            Workspace::new("3"),
            Workspace::new("4"),
        ];
        Self {
            workspaces: ws,
            current_idx: 0,
            last_idx: 0,
        }
    }

    pub fn current(&self) -> &Workspace {
        &self.workspaces[self.current_idx]
    }
    pub fn last(&self) -> &Workspace {
        &self.workspaces[self.last_idx]
    }

    pub fn switch_to(&mut self, name: &str) -> bool {
        if let Some(idx) = self.workspaces.iter().position(|w| w.name == name) {
            self.last_idx = self.current_idx;
            self.current_idx = idx;
            true
        } else {
            false
        }
    }

    pub fn switch_relative(&mut self, delta: i32) {
        let n = self.workspaces.len() as i32;
        let new = (self.current_idx as i32 + delta).rem_euclid(n) as usize;
        self.last_idx = self.current_idx;
        self.current_idx = new;
    }
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_new() {
        let w = Workspace::new("test");
        assert_eq!(w.name, "test");
    }

    #[test]
    fn test_workspace_manager_default() {
        let mgr = WorkspaceManager::new();
        assert_eq!(mgr.workspaces.len(), 4);
        assert_eq!(mgr.current().name, "1");
    }

    #[test]
    fn test_workspace_switch() {
        let mut mgr = WorkspaceManager::new();
        assert!(mgr.switch_to("3"));
        assert_eq!(mgr.current().name, "3");
        assert!(!mgr.switch_to("nonexistent"));
    }

    #[test]
    fn test_workspace_switch_relative() {
        let mut mgr = WorkspaceManager::new();
        mgr.switch_relative(1);
        assert_eq!(mgr.current().name, "2");
        mgr.switch_relative(-1);
        assert_eq!(mgr.current().name, "1");
        mgr.switch_relative(3);
        assert_eq!(mgr.current().name, "4");
        mgr.switch_relative(1);
        assert_eq!(mgr.current().name, "1");
    }
}
