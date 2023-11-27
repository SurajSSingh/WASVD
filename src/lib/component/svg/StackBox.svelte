<script lang="ts">
	import type { SVGStackObjData } from "$lib/stackAnim";

    export let data: SVGStackObjData = {
        opacity: 0,
        xPosition: 0,
        yPosition: 0,
        width: 200,
        height: 100,
        fill: "white",
        stroke: "red",
        text: "black",
        value: "...",
    };

    function stroke_color(): string{
        if (typeof data.stroke === "object"){
            return data.stroke.color
        }
        return data.stroke
    }

    function text_color(): string{
        if (typeof data.text === "object"){
            return data.text.color
        }
        return data.text
    }

    function stroke_size(): number{
        if (typeof data.stroke === "object"){
            return data.stroke.size
        }
        return 5
    }

    function font_size(): number{
        if (typeof data.text === "object"){
            return data.text.size
        }
        return data.height * 0.75
    }
</script>

<rect x="{data.xPosition}" y="{data.yPosition}" width="{data.width}" height="{data.height}" opacity={data.opacity} stroke="{stroke_color()}" fill="{data.fill}" stroke-width="{stroke_size()}"/>
<text x="{data.xPosition + data.width/2}" y="{data.yPosition + data.height/2 + 5}" fill="{text_color()}" opacity={data.opacity} font-size={font_size()} text-anchor="middle" dominant-baseline="middle" font-family="JetBrains Mono">{data.value}</text>
