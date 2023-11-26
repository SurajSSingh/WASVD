<script lang="ts">
	import {Presentation, Slide} from "$lib/animotion/components/";
	import { formatStack, type EvalResult } from "$lib/interpreter";

    export let steps: { result: EvalResult; previous: (number | bigint)[]; current: (number | bigint)[] }[] = [];
</script>

<Presentation>
    {#each steps as step, i}
    <Slide>
        <p class="font-bold text-sm">Step #{i + 1}:</p>
        <p class="text-xs">{step.result.action}</p>
        <p class="text-xs">Start: {formatStack(step.previous)}</p>
        <p class="text-xs">End: {formatStack(step.current)}</p>
    </Slide>
    {/each}
</Presentation>
<div id="stack-zone" class="h-1/3">

</div>