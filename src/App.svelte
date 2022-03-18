<script>
	import { invoke } from '@tauri-apps/api/tauri'
	import { Command, open } from '@tauri-apps/api/shell'
	import * as Realm from "realm-web"
	import { projection } from './store.js'
	import { onMount } from 'svelte'
	import { downloadDir, appDir } from "@tauri-apps/api/path"
	import { tempdir } from "@tauri-apps/api/os"
	import { copyFile, removeFile, readTextFile } from "@tauri-apps/api/fs"
	import { getMatches } from '@tauri-apps/api/cli'
	import { getVersion } from '@tauri-apps/api/app'
	import { message } from '@tauri-apps/api/dialog'
	import TailwindCSS from './TailwindCSS.svelte'
	// import { getLocalPrinters, local_printers } from "./components/GetLocalPrinters.svelte"
	import { executeCommand } from './components/ExecuteCommand.svelte'

	// UPDATER
	import { checkUpdate, installUpdate } from "@tauri-apps/api/updater"
	import { relaunch, exit } from "@tauri-apps/api/process";
	import { listen } from "@tauri-apps/api/event";

	let statusMessage = '';
	let version;
	let logList = [];
	(async () => {
		version = await getVersion()
		try {
			let {shouldUpdate, manifest} =  await checkUpdate()
			if (shouldUpdate) {
				// display dialog
				// await installUpdate()

				// download file
				manifest = manifest.body.replace(/'/ig, `"`)
				manifest = JSON.parse(manifest)
				fetch(manifest[1])
				.then((transfer) => {
					return transfer.blob()
				}).then(async (bytes) => {
					let elm = document.createElement('a')
					elm.href = URL.createObjectURL(bytes)
					elm.setAttribute('download', 'scraper_update.cab')
					elm.click()
					setTimeout(async () => {
						let download_dir = await downloadDir()
						let current_dir = await currentDir()
						let temp_dir = await tempdir()
						copyFile(`${download_dir}scraper_update.cab`, `${temp_dir}scraper_update.cab`)
							.then((result) => {
								removeFile(`${download_dir}scraper_update.cab`)
								.then(result => console.log("File deleted", result))
								.catch(error => console.error("Error deleting file", error))

								const extract = new Command("extrac32.exe", ['/Y', `${temp_dir}scraper_update.cab`, '/L', temp_dir, /* only extract app file */"Printer-Scraper.exe"])
								extract.execute()
								extract.on('close', data => {
									console.log('Extract finished', temp_dir)
									const secondInstance = new Command('binaries\\start_second_instance.cmd', [`${temp_dir}Printer-Scraper.exe`, '-p', current_dir])
									secondInstance.execute()
									secondInstance.stdout.on('data', data => {
										exit()
									})
								})
							})
							.catch((error) => {
								console.error("copyFile", error)
							})
					}, 2000)

				}).catch((error) => {
					console.error(error)
				})

				// install complete, restart app
				// await relaunch()
			}
		} catch(error) {
			console.error("Updater error:", error)
		}
	})()


	listen("tauri://update-available", function (res) {
	  console.log("New version available: ", res);

	  listen("tauri://update-status", function (res) {
		  console.log("New status: ", res);
		});
	});

	// UPDATER

	const REALM_APP_ID = "printer_scraper-drfeu"
	const realmApp = new Realm.App({id: REALM_APP_ID})
	const company = "Fenix MFG"
	const email = "printers@fenix.com"
	const pass = "fENIX.PRINTERS!"
	let realmUser,
			isConnected = false,
			printers = [],
			mongodb = {},
			printersCollection = {},
			configCollection = {},
			config = {},
			intervalTasks,
			configLocal = {},
			localPrinters = ['Buscando printers'],
			localPrintersComponent,
			printerSelected = 'Buscando printers'


	async function getConfigLocal(){
		configLocal = await readTextFile("printer-scraper-config.json")
		configLocal = JSON.parse(configLocal)
		console.log(configLocal.name)
	}

	async function createPerformanceMonitor() {

	}

	function getLocalPrinters() {
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
			localPrinters = result
		})
	}

	async function readXML() {
		// Future read xml function
		executeCommand('run-type', [], (err, result) => {
			if (err) {
				console.error("Error ejecutando comando", err)
				return
			}
			console.log(result)
		})
	}

	async function importPerformanceMonitor() {
		let app_dir = await appDir()
		console.log(app_dir)
		executeCommand('import-template', ['C:\\performance_monitor_template.xml'], (err, result) => {
			if (err) {
				console.error("Error importando PM template", err)
				return
			}
			console.log(result)
			if (result === "The command completed successfully."){
				console.log("Template importado con exito")
			}
		})
	}

	onMount(async () => {
		getConfigLocal()
		getLocalPrinters()
		// readXML()
		// importPerformanceMonitor()
		await loginToRealm().then((user) => {
			if (user){
				realmUser = user
				initializeMongoCollections().then(() => {
					// startInterval()
				})
			}
		})

		getMatches()
			.then(async (value) => {
				let temp_dir = await tempdir()
				if (!value.args.path.value) return
				const originPath = value.args.path.value
				const extractToOrigin = new Command("extrac32.exe", ['/Y', `${temp_dir}scraper_update.cab`, '/L', originPath])
				extractToOrigin.execute()
				extractToOrigin.on('close', data => {
					setTimeout(() => {
						console.log('testtt')
						let origin_path = value.args.path.value
						open(`${origin_path}Printer-Scraper.exe`)
							.then(result => {
								console.log("Origin app opened")
								exit()
							})
							.catch(error => console.error("Opening origin app", error))
					}, 60000)
				})
			})
			.catch((error) => {
				statusMessage = error
			})
	})

	async function loginToRealm() {
	  const credentials = Realm.Credentials.emailPassword(email, pass)
	  try {
	    const user = await realmApp.logIn(credentials)
	    console.info("Sesion iniciada en Realm")
	    return user
	  } catch (err) {
	    console.error("No se pudo iniciar sesion en Realm", err)
	  }
	}

	async function initializeMongoCollections() {
	  mongodb = realmApp.currentUser.mongoClient("mongodb-atlas")
	  printersCollection = mongodb.db("printer_scraper").collection("printers")
	  getPrinters()
	  config = await printersCollection.findOne({type: "config", company})
	  console.log(printers)
	}

	function startInterval() {
		intervalTasks = setInterval(() => {
			getPrinters()
			getConfig()
			console.log("Server is online")
		}, 30000)
	}

	async function getPrinters() {
    printers = await printersCollection.find({type: "printer", company}, { 
      projection: $projection
    })
  }

  async function getConfig() {
    config = await printersCollection.findOne({type : "config", company})
    checkConfig()
  }

  function checkConfig() {
  	console.info(config)
    if (config.update_printers) {
      localStorage.printers = JSON.stringify(printers)
    }
    if (config.update_count) {
      startScraper()
    }
    if (config.update_count_alone.length >= 1) {
      console.log('This', config.update_count_alone)
      getPrintersAlone()
    }
  }

	function callCommand() {
		invoke('get_counters', { name: 'Sharp' })
		.then((message) => console.log(message))
	}

	const getTimestamp = () => {
		let date = new Date()
		return date.toLocaleString()
		//return `${date.getDate()}-${date.getMonth()}-${date.getFullYear()}T${date.getHours()}:${date.getMinutes()}:${date.getSeconds()}`
	}

</script>
<TailwindCSS/>


<div class="container p-2 h-screen">
	{#if true}
		<select bind:value={printerSelected}>
			{#each localPrinters as printer}
				<option value={printer}>
					{printer}
				</option>
			{/each}
		</select>
		<input bind:value={configLocal.name} type="text">
	{/if}
	<p>Información de status:</p>
	<p>{statusMessage}</p>
	<div class="relative bg-white h-1/4 w-100 p-2 overflow-auto rounded-sm">
		<ul class="list-none">
			{#each logList as _log}
			<li>[{getTimestamp()}] {_log}</li>
			{/each}
		</ul>
	</div>

	<footer class="text-gray-600 body-font bottom-0 fixed">
	  <div class="container px-5 py-8 mx-auto flex items-center sm:flex-row flex-col">
	    <a class="flex title-font font-medium items-center md:justify-start justify-center text-gray-900">
	      <img src="logo.png" class="w-14 h-14 text-white p-2 bg-indigo-100 rounded-full"/>
	      <span class="ml-3 text-xl">Printer Scraper</span>
	    </a>
	    <p class="text-sm text-gray-500 sm:ml-4 sm:pl-4 sm:border-l-2 sm:border-gray-200 sm:py-2 sm:mt-0 mt-4">2021 Printer Scraper v{version}
	    </p>
	  </div>
	</footer>
</div>
<style>
	
</style>
