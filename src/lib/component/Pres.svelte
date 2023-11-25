<script lang="ts">
	import {Presentation, Slide} from "$lib/animotion/components/";
	import { formatStack, type EvalResult } from "$lib/interpreter";

    export let steps: { result: EvalResult; previous: (number | bigint)[]; current: (number | bigint)[] }[] = [];
</script>

<Presentation>
    {#each steps as step, i}
    <Slide>
        <p class="font-bold">Step #{i + 1}:</p>
        <p>{step.result.action}</p>
        <p>Start: {formatStack(step.previous)}</p>
        <p>End: {formatStack(step.current)}</p>
    </Slide>
    {/each}
</Presentation>