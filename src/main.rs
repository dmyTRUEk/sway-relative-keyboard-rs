//! Whenever the user hovers over a window, write its name to stdout.

extern crate i3ipc;

use {
    std::{
        str,
        thread,
        process::Command,
        collections::HashMap,
        sync::{Arc, Mutex},
    },
    json,
    i3ipc::{
        event::Event,
        I3EventListener,
        Subscription,
    },
};



// fn run_command(connection: &mut I3Connection, command: &str) -> &'static str {
//     let tmp = connection
//         .run_command(command)
//         .expect("failed to send command");
//     println!("{:?}", connection.get_outputs().expect("failed to get outputs"));
//     let outcomes = tmp.outcomes;
//     for outcome in outcomes {
//         println!("{:?}", outcome);
//         if outcome.success {
//             println!("success");
//         } else {
//             println!("failure");
//             if let Some(e) = outcome.error.as_ref() {
//                 println!("{}", e);
//             }
//         }
//     }
//     ""
// }
// run_command(&mut connection, "floating enable");



/// swaymsg input type:keyboard xkb_switch_layout 0
fn set_current_layout(new_layout_id: usize) {
    println!("setting layout id = {}", new_layout_id);
    let command_to_execute = format!("swaymsg input type:keyboard xkb_switch_layout {}", new_layout_id);
    let _output = Command::new("sh")
        .arg("-c")
        .arg(command_to_execute)
        .output()
        .expect("failed to execute process");
    // println!("{:?}", output);
    // let stdout_as_str = bytes_to_str(&output.stdout);
    // println!("{}", stdout_as_str);
}



fn bytes_to_str(bytes: &Vec<u8>) -> &str {
    let s = match str::from_utf8(bytes) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    s
}



fn get_all_layouts() -> Vec<String> {
    let command_to_execute = "swaymsg --type get_inputs | jq --raw-output '[ .[] | select(.type == \"keyboard\") | .xkb_layout_names ] | first'";
    let output = Command::new("sh")
        .arg("-c")
        .arg(command_to_execute)
        .output()
        .expect("failed to execute process");
    // println!("{:?}", output);
    let stdout_as_str = bytes_to_str(&output.stdout);
    // println!("{}", stdout_as_str);
    let language_json_array = json::parse(stdout_as_str).expect("failed to parse json");
    // println!("{:?}", language_json_array);
    let mut language_str_array: Vec<String> = vec![];
    for i in 0..language_json_array.len() {
        let language_str = language_json_array[i].as_str().expect("failed to unwrap json str");
        // println!("language_str = {}", language_str);
        language_str_array.push(language_str.to_string());
    }
    // println!("{:#?}", language_str_array);
    language_str_array
}



/// swaymsg --type get_inputs | jq --raw-output '[ .[] | select(.type == "keyboard") | .xkb_active_layout_name | select(contains("English (US)") | not) ] | first // "en"'
/// is from:
/// https://github.com/Alexays/Waybar/pull/85#issuecomment-525223382
/// https://github.com/Sunderland93/dotfiles-sway/blob/master/.config/waybar/modules/kblayout
fn get_current_layout() -> String {
    let command_to_execute = "swaymsg --type get_inputs | jq --raw-output '[ .[] | select(.type == \"keyboard\") | .xkb_active_layout_name ]'";
    let output = Command::new("sh")
        .arg("-c")
        .arg(command_to_execute)
        .output()
        .expect("failed to execute process");
    // println!("{:?}", output);
    let stdout_as_str = bytes_to_str(&output.stdout);
    // println!("{}", stdout_as_str);
    let language_json_array = json::parse(stdout_as_str).expect("failed to parse json");
    // println!("{:?}", language_json_array);
    let language_str = language_json_array[0].as_str().expect("failed to unwrap json::short::Short");
    // println!("{:?}", language_str);
    language_str.to_string()
}



/// swaymsg --type get_inputs | jq --raw-output '[ .[] | select(.type == "keyboard") | .xkb_active_layout_name | select(contains("English (US)") | not) ] | first // "en"'
/// is from:
/// https://github.com/Alexays/Waybar/pull/85#issuecomment-525223382
/// https://github.com/Sunderland93/dotfiles-sway/blob/master/.config/waybar/modules/kblayout
fn keyboard_changer_listener() -> String {
    let command_to_execute = "swaymsg --type subscribe --raw '[\"input\"]' | jq --raw-output --unbuffered 'select(.change == \"xkb_layout\") | .input.xkb_active_layout_name'";
    let output = Command::new("sh")
        .arg("-c")
        .arg(command_to_execute)
        .output()
        .expect("failed to execute process");
    // println!("{:?}", output);
    let stdout_as_str = bytes_to_str(&output.stdout);
    // println!("{}", stdout_as_str);
    let language_str = stdout_as_str.trim();
    // println!("{:?}", language_str);
    language_str.to_string()
}



fn main() {
    let all_layouts = get_all_layouts();
    assert!(all_layouts.len() > 0);
    // println!("all_layouts = {:#?}", all_layouts);

    // by default, set keyboard layout 0
    set_current_layout(0);

    let mut i3listener = I3EventListener::connect().expect("failed to connect to I3EventListener");
    i3listener
        .subscribe(&[Subscription::Window])
        .expect("failed to subscribe");

    let map_window_id_to_kblayout_id: HashMap<i64, String> = HashMap::new();
    let mut _current_window_id: i64 = -1;
    let mut current_layout_name: String;

    let arc_mutex_map = Arc::new(Mutex::from(map_window_id_to_kblayout_id));
    let arc_mutex_current_window_id = Arc::new(Mutex::from(_current_window_id));

    let arc_clone_mutex_map = Arc::clone(&arc_mutex_map);
    let arc_clone_mutex_current_window_id = Arc::clone(&arc_mutex_current_window_id);
    let _handle = thread::spawn(move || {
        loop {
            let new_layout_name = keyboard_changer_listener();
            let current_window_id = arc_clone_mutex_current_window_id.lock().unwrap();
            println!("keyboard layout changed (new = {}), saving it for window_id = {}", new_layout_name, current_window_id);
            let mut map = arc_clone_mutex_map.lock().unwrap();
            map.insert(
                *current_window_id,
                new_layout_name
            );
            println!("map in 1 = {:#?}", map);
            println!();
        }
    });

    for event in i3listener.listen() {
        match event {
            Ok(Event::WindowEvent(w)) => {
                let mut current_window_id = arc_mutex_current_window_id.lock().unwrap();
                let mut map = arc_mutex_map.lock().unwrap();

                // println!("w = {:?}", w);
                let window_name = w.container.clone().name.unwrap_or("unnamed".to_owned());
                *current_window_id = w.container.id;
                println!("focused on window_id = {}, window_name = \"{}\"", current_window_id, window_name);

                let new_layout_id = match map.get(&current_window_id) {
                    Some(this_window_layout_name) => {
                        // let current_layout_id = all_layouts.iter().position(|v| v == &current_layout_name).unwrap();
                        let new_layout_id = all_layouts.iter().position(|v| v == this_window_layout_name).unwrap();
                        // println!("current_layout_id = {}, new_layout_id = {}", current_layout_id, new_layout_id);
                        new_layout_id
                    }
                    None => {
                        current_layout_name = get_current_layout();
                        // println!("{}", current_layout_name.clone().expect("failed to unwrap layout name"));
                        map.insert(
                            *current_window_id,
                            current_layout_name
                        );
                        0
                    }
                };
                set_current_layout(new_layout_id);

                println!("map in 2 = {:#?}", map);
                
                // arc_clone_mutex_current_window_id.unlock();
            }
            Err(e) => println!("Error: {}", e),
            _ => unreachable!(),
        }
        println!();
    }

}



