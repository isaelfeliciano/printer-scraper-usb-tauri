import { readable, writable } from 'svelte/store'

export const projection = readable({
	name: 1,
})