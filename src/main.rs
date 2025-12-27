use hyprland::data::{Client, Clients, Workspaces};
use hyprland::event_listener::EventListener;
use hyprland::shared::{HyprData, HyprDataActiveOptional, HyprDataVec};
use serde::Serialize;
use std::{cell::RefCell, rc::Rc};

#[derive(Serialize, Debug)]
struct Workspace {
    active: bool,
    id: i32,
    urgent: bool,
}

fn main() -> hyprland::Result<()> {
    let active_window = Client::get_active()?.unwrap();
    let workspaces: Rc<RefCell<Vec<Workspace>>> = Rc::new(RefCell::new({
        let mut ws: Vec<Workspace> = Workspaces::get()?
            .to_vec()
            .iter()
            .map(|w| Workspace {
                active: active_window.workspace.id == w.id,
                id: w.id,
                urgent: false,
            })
            .collect();
        ws.sort_by_key(|w| w.id);
        ws
    }));
    println!("{}", serde_json::to_string(&*workspaces)?);

    let mut event_listener = EventListener::new();

    let ws = Rc::clone(&workspaces);
    event_listener.add_urgent_state_changed_handler(move |address| {
        let mut workspaces = ws.borrow_mut();
        if let Some(client) = Clients::get()
            .ok()
            .and_then(|c| c.into_iter().find(|c| c.address == address))
            && let Some(ws) = workspaces.iter_mut().find(|w| w.id == client.workspace.id)
            && !ws.active
        {
            ws.urgent = true;
        }
        println!("{}", serde_json::to_string(&*workspaces).unwrap());
    });

    let ws = Rc::clone(&workspaces);
    event_listener.add_workspace_added_handler(move |data| {
        let mut workspaces = ws.borrow_mut();
        workspaces.push(Workspace {
            active: true,
            id: data.id,
            urgent: false,
        });
        workspaces.sort_by_key(|w| w.id);
        println!("{}", serde_json::to_string(&*workspaces).unwrap());
    });

    let ws = Rc::clone(&workspaces);
    event_listener.add_workspace_deleted_handler(move |data| {
        let mut workspaces = ws.borrow_mut();
        workspaces.retain(|w| w.id != data.id);
        println!("{}", serde_json::to_string(&*workspaces).unwrap());
    });

    let ws = Rc::clone(&workspaces);
    event_listener.add_workspace_changed_handler(move |id| {
        let mut workspaces = ws.borrow_mut();
        workspaces.iter_mut().for_each(|ws| ws.active = false);
        if let Some(new) = &mut workspaces.iter_mut().find(|w| w.id == id.id) {
            new.active = true;
            new.urgent = false;
        }
        workspaces.sort_by_key(|w| w.id);
        println!("{}", serde_json::to_string(&*workspaces).unwrap());
    });

    event_listener.start_listener()?;
    Ok(())
}
