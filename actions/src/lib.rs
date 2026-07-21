//! Action execution engine — all keybind and mousebind actions.

use labwc_core::Edge;
use labwc_focus::FocusManager;
use labwc_window::View;
use labwc_workspace::WorkspaceManager;
use std::sync::Arc;
use tracing::debug;

#[derive(Debug, Clone)]
pub enum Action {
    Close,
    Minimize,
    ToggleMaximize,
    ToggleFullscreen,
    ToggleDecorations,
    ToggleAlwaysOnTop,
    ToggleAlwaysOnBottom,
    ToggleShade,
    ToggleOmnipresent,
    SnapToEdge(Edge),
    MoveToEdge(Edge),
    GrowToEdge(Edge),
    ShrinkToEdge(Edge),
    MoveRelative(i32, i32),
    Focus,
    Raise,
    Lower,
    Execute(String),
    ShowMenu(String),
    RootMenu,
    ClientMenu,
    GoToDesktop(String),
    SendToDesktop(String),
    WorkspaceSwitch(i32),
    Exit,
    Reconfigure,
    SessionLock,
    ToggleShowDesktop,
    CycleWindows(String),
    None_,
}

pub fn execute(
    action: &Action,
    view: Option<&Arc<View>>,
    focus: &mut FocusManager,
    workspace: &mut WorkspaceManager,
) {
    match action {
        Action::Close => {
            if let Some(v) = view {
                debug!("Close view {}", v.id);
            }
        }
        Action::ToggleMaximize => {
            if let Some(v) = view {
                v.toggle_maximize();
            }
        }
        Action::ToggleFullscreen => {
            if let Some(v) = view {
                v.toggle_fullscreen();
            }
        }
        Action::SnapToEdge(edge) => {
            if let Some(v) = view {
                if let Some(output) = v.output.lock().as_ref() {
                    v.snap_to_edge(*edge, output);
                }
            }
        }
        Action::MoveRelative(dx, dy) => {
            if let Some(v) = view {
                v.move_relative(*dx, *dy);
            }
        }
        Action::Focus => {
            if let Some(v) = view {
                focus.focus_view(v, true);
            }
        }
        Action::Execute(cmd) => {
            let _ = labwc_util::spawn_async_no_shell(cmd);
        }
        Action::Exit => debug!("Exit requested"),
        Action::Reconfigure => debug!("Reconfigure requested"),
        Action::GoToDesktop(name) => {
            workspace.switch_to(name);
        }
        Action::SendToDesktop(name) => {
            debug!("SendToDesktop {}", name);
        }
        Action::WorkspaceSwitch(delta) => {
            workspace.switch_relative(*delta);
        }
        _ => debug!("Action {:?} not yet implemented", action),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_default_none() {
        let a = Action::None_;
        assert!(matches!(a, Action::None_));
    }

    #[test]
    fn test_action_execute() {
        let a = Action::Execute("true".into());
        assert!(matches!(a, Action::Execute(_)));
    }

    #[test]
    fn test_action_clone() {
        let a = Action::Close;
        let b = a.clone();
        assert!(matches!(b, Action::Close));
    }
}
