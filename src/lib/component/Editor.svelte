<script lang="ts">
	import CodeMirror from 'svelte-codemirror-editor';
	import { wast } from '@codemirror/lang-wast';
	import * as command from '$lib/bindings';
	import {
		deserialize_number,
	} from '$lib';
	import { TabGroup, Tab } from '@skeletonlabs/skeleton';
	import InstructionBlock from './InstructionBlock.svelte';
	import {watStructure} from "$lib/store"
	import CompilerDebug from './CompilerDebug.svelte';
	import StructureView from './StructureView.svelte';

	let text: string = `(module
	(func $example (export "example") (result i32)
		i32.const 1
		i32.const 2
		i32.add
	)
)`;
	let tabSet: number = 0;
	let compError: object | null = null;
	let currentResult: "✅" | "❌" | "❔" = "❔";

	function reset(){
		watStructure.set(null);
		compError = null;
		currentResult = "❔";
	}

	async function compile() {
		reset();
		command
			.transform(text)
			.then((res) => {
				currentResult = "✅";
				console.log(res);
				watStructure.set(res);
			})
			.catch((err) => {
				currentResult = "❌";
				compError = err;
			});
	}

</script>


<div class="h-full overflow-auto">
	<button on:click={compile} class="bg-primary-400 p-2">Compile</button>
	<TabGroup>
		<Tab bind:group={tabSet} name="tab1" value={0}>
			{currentResult}Editor
		</Tab>
		<Tab bind:group={tabSet} name="tab2" value={1}>Exported View</Tab>
		<Tab bind:group={tabSet} name="tab3" value={2}>Raw Structure</Tab>
		<!-- Tab Panels --->
		<svelte:fragment slot="panel">
			{#if tabSet === 0}
				<CodeMirror bind:value={text} on:change={() => currentResult="❔"} lang={wast()} class=" bg-slate-100 text-black" />
			{:else if tabSet === 1}
				<StructureView />
			{:else if tabSet === 2}
				<CompilerDebug {compError}/>
			{/if}
		</svelte:fragment>
	</TabGroup>
</div>
