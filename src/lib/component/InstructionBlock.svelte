<script lang="ts">
	import { instruction_in_plain_english } from '$lib';
	import type { SerializedInstructionNode } from '$lib/bindings';

	export let nodes: SerializedInstructionNode[];
</script>

<ul role="list" class="list list-disc list-inside m-1 pl-2">
	{#each nodes as node}
		{#if 'NonBlock' in node}
			<li>{instruction_in_plain_english(node.NonBlock)}</li>
		{:else if 'SingleBlock' in node}
			<li>{node.SingleBlock.is_loop ? 'Loop' : 'Block'} {node.SingleBlock.label}:</li>
			<li><svelte:self nodes={node.SingleBlock.inner_nodes} /></li>
		{:else if 'ConditionalBlock' in node}
			<li>If {node.ConditionalBlock.label} value on stack is non-zero - Then:</li>
			<li><svelte:self nodes={node.ConditionalBlock.then_nodes} /></li>
			<li>If {node.ConditionalBlock.label} value on stack is 0 - Else:</li>
			<li><svelte:self nodes={node.ConditionalBlock.else_nodes} /></li>
		{/if}
	{/each}
</ul>
