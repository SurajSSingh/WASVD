<script lang="ts">
    import {watStructure, compErr} from "$lib/store"

</script>

<section class=" bg-slate-100">
    {#if $compErr}
        <p class=" bg-error-700">{JSON.stringify($compErr, null, 4)}</p>
    {:else if $watStructure}
        <p class=" bg-success-700">{JSON.stringify($watStructure, null, 4)}</p>
    {:else}
        <p class=" text-black">No Output (Compile code first)</p>
    {/if}
</section>