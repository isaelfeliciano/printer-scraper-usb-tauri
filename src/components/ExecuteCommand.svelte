<script context="module">
	import { Command } from '@tauri-apps/api/shell'
	export async function executeCommand(program, parameters, callback) {
		const command = await new Command(program, parameters).execute()
		if (command.stderr) {
			callback(command.stderr, null)
			return
		}
		callback(null, command.stdout)
	}

</script>