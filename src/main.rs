//! Whenever the user hovers over a window, write its name to stdout.

extern crate swayipc;

use {
    std::collections::HashMap,
    swayipc::reply::{
        Event,
        // InputEvent,
    },
    swayipc::{
        Connection,
        EventType,
        Fallible,
    },
};



fn get_input_ids(c: &mut Connection) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    match c.get_inputs() {
        Ok(inputs) => {
            for i in inputs {
                ids.push(i.identifier);
            }
        }
        _ => {}
    }
    ids
}



/// this is fork of:
/// https://github.com/house-of-vanity/swkb/blob/master/src/main.rs
/// which license is WTFPL, so i copy it :)
fn main() -> Fallible<()> {
    

    let default_kblayout_id: i64 = 0;


    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let file = xdg_dirs.place_config_file("swkb.lock").unwrap();
    let instance_a = single_instance::SingleInstance::new(file.to_str().unwrap()).unwrap();
    if !instance_a.is_single() {
        println!("Only one instance of sway-rkbd-rs at a time is allowed, exiting this instance.");
        std::process::exit(1);
    }
    let mut connection = Connection::new()?;
    let inputs = get_input_ids(&mut connection);
    let mut map_window_id_to_layout_id: HashMap<i64, i64> = HashMap::new();
    // let subs = [EventType::Input, EventType::Window];
    let mut current_window_id: i64 = 0;
    let mut current_layout_id: i64 = default_kblayout_id;
    for event in connection.subscribe(&[EventType::Input, EventType::Window])? {
        // println!("event = {:?}", event);
        match event {
            Ok(Event::Input(event_input)) => {
                let all_layouts = event_input.input.xkb_layout_names;
                let current_layout_name = event_input.input.xkb_active_layout_name.unwrap_or("none".to_string());
                let new_layout_id = all_layouts.iter().position(|v| *v == current_layout_name).unwrap() as i64;
                let is_printed_similar_event: bool = new_layout_id == current_layout_id;
                current_layout_id = new_layout_id;
                map_window_id_to_layout_id.insert(
                    current_window_id,
                    current_layout_id
                );
                if !is_printed_similar_event {
                    println!("event_input occured");
                    // println!("all_layouts = {:#?}", all_layouts);
                    println!("current_layout_name = {}", current_layout_name);
                    println!("map in 1 = {:#?}", map_window_id_to_layout_id);
                    println!();
                }
            }
            Ok(Event::Window(event_window)) => match event_window.change { swayipc::reply::WindowChange::Focus => {
                let new_window_id = event_window.container.id;
                let is_printed_similar_event: bool = new_window_id == current_window_id;
                current_window_id = new_window_id;
                let current_window_name = event_window.container.name.unwrap_or("none".to_string());
                let new_layout_id: i64 = match map_window_id_to_layout_id.get(&current_window_id) {
                    Some(this_window_layout_id) => {
                        *this_window_layout_id
                    }
                    None => {
                        let new_layout_id = default_kblayout_id;
                        map_window_id_to_layout_id.insert(
                            current_window_id,
                            new_layout_id
                        );
                        new_layout_id
                    }
                };
                let mut connection = Connection::new()?;
                // set current input layout
                for input in &inputs {
                    connection.run_command(format!(
                        "input {} xkb_switch_layout {}",
                        input,
                        new_layout_id
                    ))?;
                }
                if !is_printed_similar_event {
                    println!("event_window focus occured");
                    println!("focused on window_id = {}, window_name = \"{}\"", current_window_id, current_window_name);
                    println!("map in 2 = {:#?}", map_window_id_to_layout_id);
                    println!();
                }
            } _ => {} }
            Err(e) => {
                println!("Error: {}", e);
            }
            _ => {}
        }
    }
    unreachable!();
}



