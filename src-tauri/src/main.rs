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

fn main() {
	tauri::Builder::default()
	.invoke_handler(tauri::generate_handler![get_counters])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
