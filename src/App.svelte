<script>
	import { invoke } from '@tauri-apps/api/tauri'
	import { Command, open } from '@tauri-apps/api/shell'
	import * as Realm from "realm-web"
	import { projection } from './store.js'
	import { onMount } from 'svelte'
	import { downloadDir, appDir, resourceDir, join } from "@tauri-apps/api/path"
	import { tempdir } from "@tauri-apps/api/os"
	import { copyFile, removeFile, readTextFile, writeFile } from "@tauri-apps/api/fs"
	import { getMatches } from '@tauri-apps/api/cli'
	import { getVersion } from '@tauri-apps/api/app'
	import { message } from '@tauri-apps/api/dialog'
	import { writeText } from '@tauri-apps/api/clipboard'
	import { appWindow } from '@tauri-apps/api/window'
	import TailwindCSS from './TailwindCSS.svelte'
	// import { getLocalPrinters, local_printers } from "./components/GetLocalPrinters.svelte"
	import { executeCommand } from './components/ExecuteCommand.svelte'

	// UPDATER
	import { checkUpdate, installUpdate } from "@tauri-apps/api/updater"
	import { relaunch, exit } from "@tauri-apps/api/process";
	import { listen } from "@tauri-apps/api/event";

	let selectedWindow = appWindow.label;
	const windowMap = {
		[selectedWindow]: appWindow
	}

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
			printerSelected = 'Buscando printers',
			app_dir = "",
			resource_dir = "",
			pathTemplateXml = "",
			pathCounterJSON = "",
			counter = {},
			thisPrinterFromRemote = {}


	async function getConfigLocal(){
		app_dir = await appDir()
		resource_dir = await resourceDir()
		pathTemplateXml = await join(resource_dir, "assets", "performance_monitor_template.xml")
		pathCounterJSON = await join(resource_dir, "assets", "counter.json")
		console.log(resource_dir)
		console.log(app_dir)
		configLocal = await readTextFile(resource_dir+"assets\\printer-scraper-config.json")
		configLocal = await readTextFile(resource_dir+"assets\\printer-scraper-config.json")
		counter = await readTextFile(pathCounterJSON)
		configLocal = JSON.parse(configLocal)
		counter = JSON.parse(counter)
		console.log("Friendly name: ", configLocal.name)
		console.log(counter)
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
		executeCommand('import-template', ["import", "Printer_Counter", "-xml", "C:\\performance_monitor_template.xml"], (err, result) => {
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

	function copyToClipboard(text) {
		writeText(text)
			.then(result => openTemplateXml())
			.catch(err => console.error(err))
	}
	async function openTemplateXml() {
		executeCommand('notepad', [pathTemplateXml], (err, result) => {
			if (err) {
				console.error("Error abriendo template", err)
				return
			}
			console.log(result)
		})
	}

	async function uploadCounter() {
		console.log("uploadCounter")
		let { name, company, type } = configLocal
		// let { counter } = counter.actual
		try {
			await printersCollection.updateOne({name, company},
			{
				"$set": {
					name,
					company,
					type,
					counter: counter.actual,
					date_collected: new Date()
				}
			}, {"upsert": true})
		} catch(err) {
			console.log(err)
		}
	}

	function getThisPrinterFromRemote() {
		console.log("getThisPrinterFromRemote")
		return new Promise(async (resolve, reject) => {
			let { name, company } = configLocal
			try {
				thisPrinterFromRemote = await printersCollection.findOne({name, company})
				console.log("Remote printer", thisPrinterFromRemote)
				resolve("Informacion remota obtenida con exito")
			} catch {
				reject("Error obteniendo informacion remota")
			}
		})
	}

	async function saveFriendlyName() {
		try {
			await writeFile({
				contents: JSON.stringify(configLocal),
				path: resource_dir+"assets\\printer-scraper-config.json"
			})
			log("Nuevo nombre guardado con exito")
		} catch(err) {
				console.error("Error guardando config local", err)
				log(`Error guardando counter config local Error: ${err}`)
		}
	}

	async function setCounter() {
		try {
			await writeFile({
				contents: JSON.stringify(counter),
				path: pathCounterJSON
			})
			log("Contador guardado con exito")
		} catch(err) {
				console.error("Error guardando counter", err)
				log(`Error guardando counter Error: ${err}`)
		}
	}

	function log(text) {
		logList = [...logList, text]
	}

	onMount(async () => {
		// await getThisPrinterFromRemote()
		// readXML()
		// importPerformanceMonitor()
		setTimeout(() => {
			// windowMap[selectedWindow].hide()
		}, 3000)

		getMatches()
			.then(async (value) => {
				console.log(value)
				let temp_dir = await tempdir()
				// if (value.args.upload.occurrences >= 1) {
				if (true) {
					// Connect Mongo
					// Get remote info
					// Get local info
					// Compare both counter, if local is greater then remote update remote and viceversa
					// Close
					await loginToRealm().then((user) => {
						if (user){
							realmUser = user
							initializeMongoCollections().then(() => {
								getConfigLocal()
								getLocalPrinters()
								// startInterval()
							})
						} else {
							console.log("Secion no iniciada en MongoDB")
						}
					})
					await getConfigLocal()
					await getThisPrinterFromRemote()
					let remoteCounter = (thisPrinterFromRemote != null) ? thisPrinterFromRemote.counter : 0 
					if (counter.actual > remoteCounter || thisPrinterFromRemote == null) {
						try {
							await uploadCounter()
						} catch(err) {
							console.error("Error subiendo counter")
						}
					} else if (counter.actual < remoteCounter) {
						counter.actual = remoteCounter
						counter.base = remoteCounter
						try {
							writeFile({
								contents: JSON.stringify(counter),
								path: pathCounterJSON
							}) 
						} catch(err) {
								console.error("Error guardando counter JSON", err)
						}
					}
				}
				if (!value.args.path.value) return
				const originPath = value.args.path.value
				const extractToOrigin = new Command("extrac32.exe", ['/Y', `${temp_dir}scraper_update.cab`, '/L', originPath])
				extractToOrigin.execute()
				extractToOrigin.on('close', data => {
					setTimeout(() => {
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
	{#if configLocal.first_time == true}
		<div class="mb-4">
			<label class="block text-gray-700 text-sm font-bold mb-1" for="printername">Nombre del printer</label>
			<select bind:value={printerSelected} id="printername" class="shadow border rounded py-2 px-3 text-gray-700 leading-tight focus:outline-none">
				{#each localPrinters as printer}
					<option value={printer}>
						{printer}
					</option>
				{/each}
			</select>
			<button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 shadow rounded inline-flex" on:click={copyToClipboard(printerSelected)}>
				<svg class="fill-current w-4 h-4 mr-2" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20"><path d="M21 10v10a1 1 0 0 1-1 1H10a1 1 0 0 1-1-1V10a1 1 0 0 1 1-1h10a1 1 0 0 1 1 1zM6 14H5V5h9v1a1 1 0 0 0 2 0V4a1 1 0 0 0-1-1H4a1 1 0 0 0-1 1v11a1 1 0 0 0 1 1h2a1 1 0 0 0 0-2z"/></svg>
				<span>+ </span>
				<svg class="fill-current w-4 h-4 mr-2" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20"><path fill-rule="evenodd" clip-rule="evenodd" d="M2.402 9.493A3 3 0 0 1 5.36 6h9.945a3 3 0 0 1 2.959 2.507l1.721 10.329A1 1 0 0 1 19 20H6.694a3 3 0 0 1-2.959-2.507l-1.333-8zM5.36 8a1 1 0 0 0-.987 1.164l1.334 8a1 1 0 0 0 .986.836H17.82l-1.528-9.164A1 1 0 0 0 15.306 8H5.36z" /><path fill-rule="evenodd" clip-rule="evenodd" d="M17 9a1 1 0 0 1 1-1h3a1 1 0 0 1 .98 1.196l-2 10a1 1 0 0 1-1.96-.392L19.78 10H18a1 1 0 0 1-1-1zM2.01 6.753A3 3 0 0 1 5 4h6a3 3 0 0 1 3 3 1 1 0 1 1-2 0 1 1 0 0 0-1-1H5a1 1 0 0 0-.996.927.803.803 0 0 0 .01.072l.472 2.837a1 1 0 1 1-1.972.328L2.04 7.328a2.326 2.326 0 0 1-.03-.575z"/></svg>
			</button>
		</div>
		<button class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded hidden" on:click={importPerformanceMonitor}>Import PM</button>

		<label class="block text-gray-700 text-sm font-bold mb-1" for="friendlyname">Nombre en base de datos</label>
		<input bind:value={configLocal.name} type="text" class="shadow border rounded py-2 px-3 text-gray-700 leading-tight focus:outline-none" id="friendlyname">
		<button class="w-48 bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded" on:click="{saveFriendlyName}">Guardar nombre</button>

		<label class="block text-gray-700 text-sm font-bold mb-1" for="friendlyname">Contador</label>
		<input bind:value={counter.actual} type="number" class="shadow border rounded py-2 px-3 text-gray-700 leading-tight focus:outline-none" id="friendlyname">
		<button class="w-48 bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded" on:click="{setCounter}">Establecer contador</button>

	{/if}
	<p>Información de status:</p>
	<p>{statusMessage}</p>
	<div class="shadow-inner relative bg-white h-1/4 w-100 p-2 overflow-auto rounded-sm">
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
	ul {
	    -moz-transform: rotate(180deg);
	    -webkit-transform: rotate(180deg);
	    transform: rotate(180deg);
	}
	ul > li {
	    -moz-transform: rotate(-180deg);
	    -webkit-transform: rotate(-180deg);
	    transform: rotate(-180deg);
	}
</style>
