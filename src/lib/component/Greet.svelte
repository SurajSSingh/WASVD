<script>
	import { invoke } from '@tauri-apps/api/tauri';

	let name = '';
	let greetMsg = '';
	let testMsg = '';

	async function greet() {
		greetMsg = await invoke('greet', { name });
		// testMsg = await invoke('testno');
		testMsg = await invoke('test');
	}
</script>

<div>
	<p>{testMsg}</p>
	<input
		id="greet-input"
		placeholder="Enter a name..."
		bind:value={name}
		class=" bg-primary-100 text-black"
	/>
	<button on:click={greet} class="bg-primary-400">Greet</button>
	<button on:click={greet} class="bg-primary-400">Gree2</button>
	<p>{name}</p>
	<p>{greetMsg}</p>
</div>
