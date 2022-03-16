<script context="module">
	import { executeCommand } from './ExecuteCommand.svelte'
	export var local_printers = ['Buscando printers...']
	export function getLocalPrinters() {
		executeCommand('wmic', ['printer', 'list', 'brief'], (err, result) => {
			if (err) {
				console.error("Error ejecutando comando", err)
				return
			}
			result = result.split('\n')
			result.pop()
			result.shift()
			result = result.map(line => {
				return line.replace('\r', '')
					.replace(/^\s*|\s*$/g, '')
					.replace(/\s{2,}/g, ',').split(',')[0]
			})
			local_printers = result
			console.log(local_printers)
		})
	}
</script>