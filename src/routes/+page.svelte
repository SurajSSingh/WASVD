<script>
	import Editor from '$lib/component/Editor.svelte';
	import Visualizer from '$lib/component/Visualizer.svelte';
</script>

<div class="h-screen w-full">
	<h1 class="text-center">Welcome to WAT Stack Visual Debugger</h1>
	<div class="grid grid-cols-2 h-full">
		<div class="h-full overflow-auto">
			<Editor />
		</div>
		<div class="h-full overflow-auto">
			<Visualizer/>
		</div>
	</div>
</div>
