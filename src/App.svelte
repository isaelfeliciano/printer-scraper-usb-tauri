<script>
	import { invoke } from '@tauri-apps/api/tauri'
	import { Command, open } from '@tauri-apps/api/shell'
	import * as Realm from "realm-web"
	import { projection } from './store.js'
	import { onMount } from 'svelte'
	import { downloadDir, appDir, resourceDir, join } from "@tauri-apps/api/path"
	import { tempdir } from "@tauri-apps/api/os"
	import { copyFile, removeFile, readTextFile, writeFile, createDir } from "@tauri-apps/api/fs"
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
			// app_dir = "",
			resource_dir = "",
			pathTemplateXml = "",
			pathCounterJSON = "",
			counter = {},
			thisPrinterFromRemote = {},
			closeWhenFinish = false


	async function getConfigLocal(){
		// app_dir = await appDir()
		resource_dir = await resourceDir()
		pathTemplateXml = await join(resource_dir, "assets", "performance_monitor_template.xml")
		pathCounterJSON = await join(resource_dir, "assets", "counter.json")
		console.log(resource_dir)
		// console.log(app_dir)
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
				console.log("Template importado con éxito")
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
		let { name, company, type, usb } = configLocal
		// let { counter } = counter.actual
		try {
			await printersCollection.updateOne({name, company},
			{
				"$set": {
					name,
					company,
					type,
					usb,
					counter: counter.actual,
					date_collected: new Date()
				}
			}, {"upsert": true})
			log("Counter actualizado con éxito", true)
			configLocal.last_time_upload = (new Date()).toJSON().slice(0, 10)
			try {
				await writeFile({
					contents: JSON.stringify(configLocal),
					path: resource_dir+"assets\\printer-scraper-config.json"
				})
				log("Fecha ultimo upload actualizado", true)
			} catch(err) {
					console.error("Error actualizando fecha ultimo upload", err)
					log(`Error actualizando fecha ultimo upload: ${err}`)
					if(closeWhenFinish)
						exit()
			}
		} catch(err) {
			console.log(err)
			log(`[ERROR]: Actualizando counter, ${err}`, true)
			if(closeWhenFinish)
				exit()
		}
	}

	function getThisPrinterFromRemote() {
		console.log("getThisPrinterFromRemote")
		return new Promise(async (resolve, reject) => {
			let { name, company } = configLocal
			try {
				thisPrinterFromRemote = await printersCollection.findOne({name, company})
				console.log("Remote printer", thisPrinterFromRemote)
				resolve("Informacion remota obtenida con éxito")
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
			log("Nuevo nombre guardado con éxito", true)
		} catch(err) {
				console.error("Error guardando config local", err)
				log(`Error guardando counter config local Error: ${err}`)
		}
	}

	async function setCounter() {
		counter.base = counter.actual
		counter.old = counter.new = 0
		try {
			await writeFile({
				contents: JSON.stringify(counter),
				path: pathCounterJSON
			})
			log("Contador guardado con éxito", true)
		} catch(err) {
				console.error("Error guardando counter", err)
				log(`Error guardando counter Error: ${err}`)
		}

		if (!configLocal.first_time && counter.actual > thisPrinterFromRemote.counter) {
			await uploadCounter()
		} else {
			log("Contador no actualizado", true)
		}
	}
	async function saveConfiguration() {
		try {
			configLocal.first_time = false
			await writeFile({
				contents: JSON.stringify(configLocal),
				path: resource_dir+"assets\\printer-scraper-config.json"
			})
			log("Configuracion guardada con éxito")
		} catch(err) {
				console.error("Error guardando config local", err)
				log(`Error guardando config local Error: ${err}`)
		}

		if (counter.actual > thisPrinterFromRemote.counter) {
			await uploadCounter()
			exit()
		} else {
			log("Contador no actualizado", true)
		}
	}

	async function start() {
		await loginToRealm().then((user) => {
			if (user){
				realmUser = user
				log("Sesión iniciada en DB", true)
				initializeMongoCollections().then(() => {
					getConfigLocal()
					// startInterval()
				})
			} else {
				console.log("Sesión no iniciada en MongoDB")
				log("Sesión no iniciada en DB", true)
				if (closeWhenFinish)
					exit()
			}
		})
		.catch(err => {
			console.log("Sesión no iniciada en MongoDB")
			log("Sesión no iniciada en DB", true)
			if (closeWhenFinish)
				exit()
		})
		await getConfigLocal()
		if (configLocal.name === "N/A") return // Printer not set yet nor in DB
		await getThisPrinterFromRemote()
		let remoteCounter = (thisPrinterFromRemote != null) ? thisPrinterFromRemote.counter : 0 
		if (counter.actual > remoteCounter || thisPrinterFromRemote == null) {
			try {
				await uploadCounter()
				log("Counter subido", true)
			} catch(err) {
				console.error("Error subiendo counter")
				log("[ERROR]: Error subiendo counter", true)
			}
		} else if (counter.actual < remoteCounter) {
			counter.actual = remoteCounter
			counter.base = remoteCounter
			try {
				writeFile({
					contents: JSON.stringify(counter),
					path: pathCounterJSON
				})
				log("Json guardado", true) 
			} catch(err) {
					console.error("Error guardando counter JSON", err)
					log(`[ERROR] guardando counter JSON: ${err}`)
			}
		}
		if (closeWhenFinish)
			exit()
	}

	async function log(text, writeLogToText) {
		logList = [...logList, text]
		if (writeLogToText) {
			let _appDir = await appDir()
			let pathLog = await join(resource_dir, 'log.txt')
			try {
				// File exist
				let logFile = await readTextFile(pathLog)
				let logFileSplit = logFile.split('\n')
				if (logFileSplit.length >= 500) { // File reach the lines limit
					try {
						writeFile({
							contents: `[${getTimestamp()}] ${text}\n`,
							path: pathLog
						})
					} catch (err) {
						console.log(`Error escribiendo log.txt : ${err}`)
					}
				} else {
					try {
						writeFile({
							contents: `${logFile}[${getTimestamp()}] ${text}\n`,
							path: pathLog
						})
					} catch (err) {
						console.log(`Error añadiendo log.txt : ${err}`)
					}
				}
			} catch(err) {
				// File does not exist
				try {
					writeFile({
						contents: `[${getTimestamp()}] ${text}\n`,
						path: pathLog
					})
				} catch (err) {
					console.log(`Error creando log.txt : ${err}`)
				}
				console.log("Error: ", err)
			}
		}
	}

	onMount(async () => {
		// await getThisPrinterFromRemote()
		// readXML()
		// importPerformanceMonitor()
		setTimeout(() => {
			// windowMap[selectedWindow].hide()
		}, 3000)

		// start()
		getMatches()
			.then(async (value) => {
				console.log(value)
				let temp_dir = await tempdir()
				if (!value.args.upload.occurrences >= 1) {
				// if (true) {
					windowMap[selectedWindow].show()
					start()
					getLocalPrinters()
					// Connect Mongo
					// Get remote info
					// Get local info
					// Compare both counter, if local is greater then remote update remote and viceversa
					// Close
					return
				}
				closeWhenFinish = true
				start()
				log("Start", true)
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
	<label class="block text-gray-700 text-sm font-bold mb-1" for="friendlyname">Contador</label>
	<input bind:value={counter.actual} type="number" class="shadow border rounded py-2 px-3 text-gray-700 leading-tight focus:outline-none" id="friendlyname">
	<button class="w-48 bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded" on:click="{setCounter}">Establecer contador</button>

	{#if configLocal.first_time == true}
		<label class="block text-gray-700 text-sm font-bold mb-1" for="friendlyname">Nombre en base de datos</label>
		<input bind:value={configLocal.name} type="text" class="shadow border rounded py-2 px-3 text-gray-700 leading-tight focus:outline-none" id="friendlyname">
		<button class="w-48 bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded" on:click="{saveFriendlyName}">Guardar nombre</button>

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

		<div class="flex">
			<div class="mr-1">
				<label class="block text-gray-700 text-sm font-bold mb-1" for="company">Compañía</label>
				<input bind:value={configLocal.company} type="text" class="shadow border rounded py-2 px-3 text-gray-700 leading-tight focus:outline-none" id="company">
			</div>
				
			<div class="mr-1">
				<label class="block text-gray-700 text-sm font-bold mb-1" for="brand">Marca</label>
				<input bind:value={configLocal.brand} type="text" class="shadow border rounded py-2 px-3 text-gray-700 leading-tight focus:outline-none" id="brand">
			</div>

			<div class="">
				<label class="block text-gray-700 text-sm font-bold mb-1" for="model">Modelo</label>
				<input bind:value={configLocal.model} type="text" class="shadow border rounded py-2 px-3 text-gray-700 leading-tight focus:outline-none" id="model">
			</div>
		</div>

		<button class="mx-auto w-full mt-4 block bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded" on:click={saveConfiguration}>Completar configuración</button>
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
