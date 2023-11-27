<script lang="ts">
	import CodeMirror from 'svelte-codemirror-editor';
	import { wast } from '@codemirror/lang-wast';
	import * as command from '$lib/bindings';
	import {
		deserialize_number,
	} from '$lib';
	import { TabGroup, Tab, Accordion, AccordionItem } from '@skeletonlabs/skeleton';
	import InstructionBlock from './InstructionBlock.svelte';
	import {watStructure, compErr} from "$lib/store"
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
	let currentResult: "✅" | "❌" | "❔" = "❔";

	function reset(){
		watStructure.set(null);
		compErr.set(null);
		currentResult = "❔";
	}

	async function compile() {
		reset();
		command
			.transform(text)
			.then((res) => {
				if("Ok" in res){
					currentResult = "✅";
					watStructure.set(res.Ok);
				}
				else {
					currentResult = "❌";
					compErr.set(res.Err)
				}
			})
			.catch((err) => {
				// Some other failure happened!
				console.error(err);
			});
	}

</script>


<div class="h-full overflow-auto">
	<h2 class="text-center">Editor</h2>
	<TabGroup>
		<Tab bind:group={tabSet} name="tab1" value={0}>Editor Tab</Tab>
		<Tab bind:group={tabSet} name="tab2" value={1}>Raw Results</Tab>
		<!-- Tab Panels --->
		<svelte:fragment slot="panel">
			{#if tabSet === 0}
			<Accordion>
				<AccordionItem open>
					<svelte:fragment slot="lead">{currentResult}</svelte:fragment>
					<svelte:fragment slot="summary">Editor</svelte:fragment>
					<svelte:fragment slot="content">
						<button on:click={compile} class="btn btn-md bg-primary-500">Compile</button>
						<CodeMirror bind:value={text} on:change={() => currentResult="❔"} lang={wast()} class=" bg-slate-100 text-black" />
					</svelte:fragment>
				</AccordionItem>
				<AccordionItem open>
					<!-- <svelte:fragment slot="lead">(icon)</svelte:fragment> -->
					<svelte:fragment slot="summary">Result</svelte:fragment>
					<svelte:fragment slot="content">
						<StructureView />
					</svelte:fragment>
				</AccordionItem>
			</Accordion>
			{:else if tabSet === 1}
				<CompilerDebug />
			{/if}
		</svelte:fragment>
	</TabGroup>
</div>
