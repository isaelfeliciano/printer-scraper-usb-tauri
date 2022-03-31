#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

#[tauri::command]
fn get_counters(name: String) -> String{
	println!("Roger");
	format!("Hey! I was invoked from JS, with this message: {}", name)
}

#[cfg(test)]
mod tests {
	use super::*;

    #[test]
    fn it_works() {
		let result = my_custom_command(String::from("test"));
        assert_eq!(result, "Hey! I was invoked from JS, with this message: test");
    }
}

use tauri::SystemTray;
use tauri::{CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem};
fn main() {

	let quit = CustomMenuItem::new("quit".to_string(), "Quit");
	let hide = CustomMenuItem::new("hide".to_string(), "Hide");
	let tray_menu = SystemTrayMenu::new()
	  .add_item(quit)
	  .add_native_item(SystemTrayMenuItem::Separator)
	  .add_item(hide);
	let system_tray = SystemTray::new().with_menu(tray_menu);
	tauri::Builder::default()
	    .system_tray(system_tray)
	    .run(tauri::generate_context!())
	    .expect("error while running tauri application");
	tauri::Builder::default()
	.invoke_handler(tauri::generate_handler![get_counters])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
