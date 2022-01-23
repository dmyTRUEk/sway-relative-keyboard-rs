//! sway-relative-keyboard-rs

extern crate swayipc;

use {
    std::collections::HashMap,
    swayipc::reply::Event,
    swayipc::{
        Connection,
        EventType,
        Fallible,
    },
};



fn get_all_input_ids(connection: &mut Connection) -> Option<Vec<String>> {
    return match connection.get_inputs() {
        Ok(inputs) => {
            let input_ids: Vec<String> = inputs.iter().map(|input| input.identifier.clone()).collect();
            Some(input_ids)
        }
        _ => { None }
    }
}



fn main() -> Fallible<()> {
    // this is default keyboard layout id, so change it, if you want it to be different
    let default_kblayout_id: i64 = 0;

    // check if only one instance
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let file = xdg_dirs.place_config_file("swkb.lock").unwrap();
    let instance_a = single_instance::SingleInstance::new(file.to_str().unwrap()).unwrap();
    if !instance_a.is_single() {
        println!("Only one instance of sway-relative-keyboard-rs at a time is allowed, exiting this instance.");
        std::process::exit(1);
    }

    let mut connection = Connection::new()?;
    let inputs = get_all_input_ids(&mut connection).unwrap();
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
                    println!("current_layout_name = {current_layout_name}");
                    println!("Event::Input -> map_window_id_to_layout_id = {:#?}", map_window_id_to_layout_id);
                    println!();
                }
            }
            Ok(Event::Window(event_window)) => match event_window.change { swayipc::reply::WindowChange::Focus => {
                let new_window_id = event_window.container.id;
                let is_printed_similar_event: bool = new_window_id == current_window_id;
                current_window_id = new_window_id;
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
                    connection.run_command(format!("input {input} xkb_switch_layout {new_layout_id}"))?;
                }
                if !is_printed_similar_event {
                    let current_window_name = event_window.container.name.unwrap_or("none".to_string());
                    println!("focused on window_id = {current_window_id}, window_name = \"{current_window_name}\"");
                    println!("Event::Window -> Focus -> map_window_id_to_layout_id = {:#?}", map_window_id_to_layout_id);
                    println!();
                }
            }, _ => {} }
            Err(e) => {
                println!("Error: {e}");
            }
            _ => {}
        }
    }
    unreachable!();
}



