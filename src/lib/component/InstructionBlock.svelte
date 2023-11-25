<script lang="ts">
	import { instruction_in_plain_english } from '$lib';
	import type {SerializedInstructionTree } from '$lib/bindings';

	export let tree: SerializedInstructionTree;
	export let index: number;

	function range(start: number, end: number): number[]{
		return Array.from(new Array(end-start+1), (_, i) => i + start);
	}

	$: current = tree.root[index];
	$: start = current.start + (index !== 0 ? 1: 0);
	$: end = current.end - (index !== 0 ? 1: 0);
	$: childrenItems = Object.values(current.children).flatMap(childIndex => {
		const child = tree.root[childIndex];
		return range(child.start, child.end);
	})
	$: instructions = range(start, end).map(n => {
		if(childrenItems.includes(n)){
			// If its in children, get child index
			if(n in current.children){
				return current.children[n]
			}
			else {
				return null
			}
		}
		else{
			return tree.array[n];
		}
	});
</script>

{#if index !== 0}
	 <li>{instruction_in_plain_english(tree.array[current.start])}</li>
{/if}
<ul role="list" class="list list-disc list-inside m-1 pl-2">
	{#each instructions as instruction}
		{#if instruction}
			 {#if typeof instruction !== "number"}
				 <li>{instruction_in_plain_english(instruction)}</li>
			 {:else}
				<svelte:self tree={tree} index={instruction}/>
			 {/if}
		{/if}
	{/each}
</ul>
{#if index !== 0}
	 <li>{instruction_in_plain_english(tree.array[current.end])}</li>
{/if}