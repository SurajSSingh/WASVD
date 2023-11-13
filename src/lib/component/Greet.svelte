<script>
	import { invoke } from '@tauri-apps/api/tauri';

	let name = '';
	let greetMsg = '';

	async function greet() {
		greetMsg = await invoke('greet', { name });
	}
</script>

<div>
	<input id="greet-input" placeholder="Enter a name..." bind:value={name} class=" bg-primary-800" />
	<button on:click={greet} class="bg-primary-400">Greet</button>
	<p>{greetMsg}</p>
</div>
