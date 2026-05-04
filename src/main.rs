use futures_lite::{StreamExt, future::block_on};
use hyprland::data::{Client, Clients, Workspaces};
use hyprland::event_listener::WorkspaceEventData;
use hyprland::event_listener::{Event, EventStream};
use hyprland::shared::{Address, HyprData, HyprDataActiveOptional, HyprDataVec};
use serde::Serialize;
use std::thread::sleep;
use std::time::Duration;

#[derive(Serialize, Debug)]
struct Workspace {
    active: bool,
    id: i32,
    urgent: bool,
}

fn main() -> hyprland::Result<()> {
    block_on(listener())
}

async fn listener() -> hyprland::Result<()> {
    let active_window = loop {
        if let Ok(Some(window)) = Client::get_active() {
            break window;
        }
        sleep(Duration::from_secs_f32(0.1));
    };
    let mut workspaces: Vec<Workspace> = Workspaces::get()?
        .to_vec()
        .iter()
        .map(|w| Workspace {
            active: active_window.workspace.id == w.id,
            id: w.id,
            urgent: false,
        })
        .collect();
    workspaces.sort_by_key(|w| w.id);
    println!("{}", serde_json::to_string(&*workspaces)?);

    let mut stream = EventStream::new();
    while let Some(event) = stream.next().await {
        if let Ok(event) = event {
            match event {
                Event::UrgentStateChanged(address) => urgent(&address, &mut workspaces),
                Event::WorkspaceAdded(data) => added(&data, &mut workspaces),
                Event::WorkspaceDeleted(data) => workspaces.retain(|w| w.id != data.id),
                Event::WorkspaceChanged(data) => changed(&data, &mut workspaces),
                _ => continue,
            }
            println!("{}", serde_json::to_string(&workspaces).unwrap());
        }
    }
    Ok(())
}

fn urgent(address: &Address, workspaces: &mut [Workspace]) {
    if let Ok(Some(client)) = Clients::get().map(|c| c.into_iter().find(|c| &c.address == address))
        && let Some(ws) = workspaces.iter_mut().find(|w| w.id == client.workspace.id)
        && !ws.active
    {
        ws.urgent = true;
    }
}

fn added(data: &WorkspaceEventData, workspaces: &mut Vec<Workspace>) {
    workspaces.push(Workspace {
        active: true,
        id: data.id,
        urgent: false,
    });
    workspaces.sort_by_key(|w| w.id);
}

fn changed(data: &WorkspaceEventData, workspaces: &mut [Workspace]) {
    workspaces.iter_mut().for_each(|ws| ws.active = false);
    if let Some(new) = &mut workspaces.iter_mut().find(|w| w.id == data.id) {
        new.active = true;
        new.urgent = false;
    }
    workspaces.sort_by_key(|w| w.id);
}
