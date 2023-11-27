<script lang="ts">
	import {Presentation, Slide} from "$lib/animotion/components/";
	import type { SerializedInstruction } from "$lib/bindings";
	import { formatStack, type EvalResult, operationFromInstruction } from "$lib/interpreter";
	import { generateStackSVGData, type StackSVGInitializer } from "$lib/stackAnim";
	import { Accordion, AccordionItem } from "@skeletonlabs/skeleton";
	import StackSvg from "./svg/StackSVG.svelte";
	import { revealDeck } from "$lib/store";

    export let steps: { result: EvalResult; previous: (number | bigint)[]; current: (number | bigint)[] }[] = [];
    export let setupData: StackSVGInitializer = {
        maxShown: 4,
        stackStart: 600,
        rectHeight: 100,
        yOutside: -1000,
        xPosition: 300,
        yTopWorking: 50,
        xWorkingOffset: 250,
        operatorSize: 0,
        defaultData: {
            xPosition: 300,
            yPosition: 600,
            height: 100,
            width: 200,
            value: 0,
        },
    };
    export const stackView: number = 5;
    function stepsToAnimation(previous: (number|bigint)[], current: (number|bigint)[], instruction: SerializedInstruction): () => Promise<void>{
        const {kind, name} = operationFromInstruction(instruction);
        return animData.buildAnimation(previous, current, kind, name);
    }

    $: animData = generateStackSVGData(setupData);

    $: animationArray = steps.map(step => {
        return {
            step,
            anim: stepsToAnimation(step.previous, step.current, step.result.instruction)
        };
    })

    let currentStep: EvalResult | null = null;
</script>

<button on:click={() => $revealDeck?.slide(0)} class="btn btn-md bg-primary-500">Back to Start</button>
<hr />
<div class="h-1/3">
{#if steps.length > 0}
<Presentation>
    {#each animationArray as {step, anim}, i}
    <Slide on:in={anim} on:in={() => currentStep = step.result} animate interactive>
        <p class="font-bold text-md p-2">Step #{i + 1}:</p>
        <p class="text-sm p-2">{step.result.action}</p>
        <p class="text-md p-3">Stack: {formatStack(step.previous, setupData.maxShown)} &RightTeeArrow; {formatStack(step.current, setupData.maxShown)}</p>
    </Slide>
    {/each}
</Presentation>
{/if}
{#if currentStep}
<Accordion>
    <AccordionItem>
        <svelte:fragment slot="summary">Locals</svelte:fragment>
        <svelte:fragment slot="content">
            <div class="table-container">
                <table class="table table-hover">
                    <thead>
                        <tr>
                            <th>Index</th>
                            <th>Name</th>
                            <th>Value</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each currentStep.locals.values as val, idx}
                            {@const nam = Object.entries(currentStep.locals.mapping).filter(kv => kv[1] === idx).map(kv => kv[0])[0]} 
                            <tr>
                                <td>{idx}</td>
                                <td>{nam}</td>
                                <td>{val}</td>
                             </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        </svelte:fragment>
    </AccordionItem>
</Accordion>
{/if}
<div id="stack-zone" class="h-1/3">
    <StackSvg 
        working1={animData.signalData.working1}  
        working2={animData.signalData.working2}  
        operator={animData.signalData.operator}  
        stacks={animData.signalData.stackArray}  
        restOf={animData.signalData.restOfStack}
        restData={setupData.defaultData ?? {}}
    />
</div>
</div>