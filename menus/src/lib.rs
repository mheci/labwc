use labwc_actions::Action;
use std::sync::Arc;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct Menu {
    pub id: String,
    pub label: String,
    pub items: Vec<MenuItem>,
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub action: MenuAction,
    pub icon: Option<String>,
    pub enabled: bool,
    pub submenu: Option<Arc<Menu>>,
}

#[derive(Debug, Clone)]
pub enum MenuAction {
    Action(Action),
    Submenu(Arc<Menu>),
    Separator,
}

#[derive(Debug)]
pub struct MenuManager {
    pub menus: Vec<Arc<Menu>>,
    pub open_menu: Option<MenuOpenState>,
}

#[derive(Debug)]
pub struct MenuOpenState {
    pub menu_id: String,
    pub position: (i32, i32),
    pub selected_index: usize,
}

impl MenuManager {
    pub fn new() -> Self {
        Self {
            menus: Vec::new(),
            open_menu: None,
        }
    }

    pub fn register(&mut self, menu: Menu) {
        self.menus.push(Arc::new(menu));
    }

    pub fn get_by_id(&self, id: &str) -> Option<Arc<Menu>> {
        self.menus.iter().find(|m| m.id == id).cloned()
    }

    pub fn open(&mut self, menu_id: &str, x: i32, y: i32) -> bool {
        if self.menus.iter().any(|m| m.id == menu_id) {
            self.open_menu = Some(MenuOpenState {
                menu_id: menu_id.into(),
                position: (x, y),
                selected_index: 0,
            });
            debug!("Menu '{}' opened at ({}, {})", menu_id, x, y);
            return true;
        }
        false
    }

    pub fn close(&mut self) {
        self.open_menu = None;
    }

    pub fn navigate(&mut self, direction: i32) {
        let menu_id = match &self.open_menu {
            Some(state) => state.menu_id.clone(),
            None => return,
        };
        let menu = self.get_by_id(&menu_id);
        if let (Some(menu), Some(state)) = (menu, &mut self.open_menu) {
            let count = menu.items.len() as i32;
            if count > 0 {
                state.selected_index =
                    ((state.selected_index as i32 + direction).rem_euclid(count)) as usize;
            }
        }
    }

    pub fn select(&mut self) -> Option<Action> {
        let state = self.open_menu.take()?;
        let menu = self.get_by_id(&state.menu_id)?;
        let item = menu.items.get(state.selected_index)?;
        match &item.action {
            MenuAction::Action(action) => Some(action.clone()),
            MenuAction::Submenu(sub) => {
                self.open_menu = Some(MenuOpenState {
                    menu_id: sub.id.clone(),
                    position: state.position,
                    selected_index: 0,
                });
                None
            }
            MenuAction::Separator => None,
        }
    }

    pub fn reconfigure(&mut self) {
        debug!("Menus reconfigured");
    }

    pub fn item_count(&self) -> usize {
        self.open_menu
            .as_ref()
            .and_then(|s| self.get_by_id(&s.menu_id))
            .map(|m| m.items.len())
            .unwrap_or(0)
    }
}

impl Default for MenuManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_register_and_get() {
        let mut mgr = MenuManager::new();
        let menu = Menu {
            id: "test".into(),
            label: "Test".into(),
            items: vec![],
        };
        mgr.register(menu);
        assert!(mgr.get_by_id("test").is_some());
        assert!(mgr.get_by_id("nonexistent").is_none());
    }

    #[test]
    fn test_menu_open_close() {
        let mut mgr = MenuManager::new();
        mgr.register(Menu {
            id: "root".into(),
            label: "Root".into(),
            items: vec![],
        });
        assert!(mgr.open("root", 100, 200));
        assert!(mgr.open_menu.is_some());
        mgr.close();
        assert!(mgr.open_menu.is_none());
    }

    #[test]
    fn test_menu_navigate() {
        let mut mgr = MenuManager::new();
        mgr.register(Menu {
            id: "root".into(),
            label: "Root".into(),
            items: vec![
                MenuItem {
                    label: "A".into(),
                    action: MenuAction::Separator,
                    icon: None,
                    enabled: true,
                    submenu: None,
                },
                MenuItem {
                    label: "B".into(),
                    action: MenuAction::Separator,
                    icon: None,
                    enabled: true,
                    submenu: None,
                },
                MenuItem {
                    label: "C".into(),
                    action: MenuAction::Separator,
                    icon: None,
                    enabled: true,
                    submenu: None,
                },
            ],
        });
        mgr.open("root", 0, 0);
        mgr.navigate(1);
        assert_eq!(mgr.open_menu.as_ref().unwrap().selected_index, 1);
        mgr.navigate(-1);
        assert_eq!(mgr.open_menu.as_ref().unwrap().selected_index, 0);
    }
}
