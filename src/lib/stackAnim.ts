import type { Resolve } from "$lib/animotion/motion/types";
import { all, signal } from "$lib/animotion/motion";
import type { Invalidator, Subscriber, TweenedOptions, Unsubscriber } from "svelte/motion";

export type StackOperationKind = "Nop" | "Pop" | "Push"  | "Unary" | "Binary";

/** Data fields used by stack objects (rect + text) */
export type SVGStackObjData = {
    opacity: 0 | 1,
    xPosition: number,
    yPosition: number,
    width: number,
    height: number,
    fill: string,
    stroke: string | {
        size: number,
        color: string,
    },
    text: string | {
        color: string,
        size: number,
    },
    value: number | bigint | string | object 
};

export type StackSVGInitializer = {
    stackStart:number, 
    rectHeight: number, 
    maxShown: number, 
    xPosition:number, 
    yOutside: number,
    yTopWorking: number,
    xWorkingOffset: number,
    operatorSize?: number,
    workingData?: Partial<SVGStackObjData>,
    operatorData?: Partial<SVGStackObjData>,
    stackData?: Partial<SVGStackObjData>,
    restData?: Partial<SVGStackObjData>,
    defaultData?: Partial<SVGStackObjData>,
};

// Type developed from "$lib/animotion/motion" with small tweaks
export type SignalResult<TweenValues> = {
    subscribe: (this: void, run: Subscriber<TweenValues>, invalidate?: Invalidator<TweenValues> | undefined) => Unsubscriber;
    to: (this: SignalResult<TweenValues>, values: Partial<TweenValues>, toOptions?: TweenedOptions<TweenValues>) => SignalResult<TweenValues>;
    reset: () => void;
    sfx: (this: SignalResult<TweenValues>, sound: string, { volume }?: {
        volume?: number | undefined;
    }) => SignalResult<TweenValues>;
    then: (resolve: Resolve) => Promise<void>;
};

function defaultData(data?: Partial<SVGStackObjData>): SVGStackObjData{
    return {opacity: 0, xPosition: 0, yPosition: 0, fill:"black", stroke: "red", text:"white", value: '', width: 200, height: 100 , ...data}
}

function getItemFromStack<T>(stack: T[], index: number): T | undefined{
    console.log("GETTING:", stack, index, stack.length - 1 - index, stack[stack.length - 1 - index])
    return stack[stack.length - 1 - index]
}

function immediate<T>(options?: TweenedOptions<T>): TweenedOptions<T> {
    return {...options, duration: 0}
}

export function generateStackSVGData(init:StackSVGInitializer){
    // Must show at least 1 more outside of "rest"
    const maxShown = Math.max(2, init.maxShown);
    // Const variable initialization
    const stackStart = init.stackStart;
    const rectHeight = init.rectHeight;
    const xPosition = init.xPosition;
    const yOutside = init.yOutside;
    const xWorkingOffset = init.xWorkingOffset;
    const yTopWorking = init.yTopWorking ?? stackStart + rectHeight * (maxShown+2);
    const operatorSize = init.operatorSize ?? rectHeight;
    const data = init.defaultData ?? {};
    data.xPosition ??= xPosition;
    data.yPosition ??= stackStart;
    data.height ??= rectHeight;
    init.workingData ??= data;
    init.operatorData ??= data;
    init.stackData ??= data;
    init.restData ??= data;

    // All SVGs:
    // 1. 3 rect for binary, unary, or pushing operations
    // 2. 1 static rectangle representing "rest of the stack"
    // 3. N for each items that is currently shown
    const topWorkingRect1 = signal(defaultData(init.workingData))
    const topWorkingRect2 = signal(defaultData(init.workingData))
    const operatorRect = signal(defaultData(init.operatorData))
    const stackRectArray = Array.from({length: maxShown}, () => signal(defaultData(init.stackData)))
    const restOfStackRect = signal<{opacity: 0 | 1}>({opacity:0})

    console.log(data, defaultData(data))

    function resetAnimation(){
        topWorkingRect1.reset();
        topWorkingRect2.reset();
        operatorRect.reset();
        stackRectArray.map(rect => rect.reset());
        restOfStackRect.reset();
    }

    function buildAnimation<T>(previous: T[], current: T[], operationKind: StackOperationKind, operation?: string, previousHasExtra?: boolean, currentHasExtra?: boolean){
        return async () => {
            resetAnimation();
            console.log(stackRectArray);
            // Offset for index, min is 0 (stack has no extra values), max is 2 (stack has two extra values at start)
            const previousOffset = Math.min(Math.max(previous.length-maxShown,0),2);
            const currentOffset = Math.min(Math.max(current.length-maxShown,0),2);
            // Slice to make sure to only see max shown value + up to 2 extra,
            // any more is not necessary for animation.
            // If there is a change, it must set hasExtra value to true.
            if(previous.length > maxShown + 2){
                previousHasExtra = true;
                previous = previous.slice(-(maxShown+2));
            }
            if(current.length > maxShown + 2){
                currentHasExtra = true;
                current = current.slice(-(maxShown+2));
            }
            if(previousHasExtra === undefined){
                previousHasExtra = previous.length > maxShown;
            }
            if(currentHasExtra === undefined){
                currentHasExtra = current.length > maxShown;
            }
            if(operation === undefined){
                operation = operationKind;
            }
            console.log(previous, current, operationKind, previousHasExtra, currentHasExtra, operation, previousOffset, currentOffset);
            const hasExtra = currentHasExtra || previousHasExtra;
            const startingExtraOpacity = previousHasExtra ? 1: 0;
            const endingExtraOpacity = currentHasExtra ? 1: 0;
            
            // Check operation kind
            if(operationKind === "Nop"){
                console.log("DO NOP");
                // No changes
                // Animation:
                // If no extra, keep all items the same on stack
                // If extra, keep all except last the same on stack
                await all(
                    ...stackRectArray.map((rect, index) => {
                        const value = current[currentOffset+index];
                        // If has value, show on stack
                        if(value){
                            return rect.to({opacity:1, yPosition: stackStart - rectHeight*index, value: value}, immediate())
                        }
                        // Otherwise hide it
                        else {
                            return rect.to({opacity: 0}, immediate())
                        }
                    }),
                    restOfStackRect.to({opacity: currentHasExtra ? 1: 0}, immediate())
                )
            }
            else if(operationKind === "Push"){
                // One push
                // [...] -> [..., X]
                // Animation:
                // If 0..N on prev stack, then just drop down pushed item
                // Otherwise, 
                // All stack items move down by 1
                const pushedPosition = hasExtra ? (maxShown-1) : (current.length-1);
                console.log("PUSHED: Extra", previousHasExtra, currentHasExtra)
                await all(
                    ...stackRectArray.map((rect,i) => {
                        const prevVal = previous[previousOffset+i];
                        if(prevVal){
                            const startHeight = stackStart - rectHeight * i;
                            // No changes in height if there is no extra
                            const endHeight = hasExtra ? stackStart - rectHeight * Math.max(i-1, 0) : startHeight;
                            const endingOpacity = i === 0 && hasExtra ? 0 : 1;
                            // Transform from previous value to current value and move box down
                            return rect
                                .to({opacity: 1, yPosition: startHeight, value: prevVal}, immediate())
                                // Fade out if at bottom
                                .to({opacity: endingOpacity, yPosition: endHeight});
                        }
                        else{
                            // Item not on stack, so hide this box 
                            // - should unreachable, but kept just in case
                            console.log("PUSHED nothing: ", i, previousOffset+i, previous)
                            return rect.to({opacity:0}, immediate());
                        }
                    }),
                    // Rest of stack
                    restOfStackRect
                        .to({opacity: startingExtraOpacity}, immediate())
                        .to({opacity: endingExtraOpacity}),
                    // Item being pushed onto the stack
                    topWorkingRect1
                        .to({opacity: 0, yPosition: yOutside, value: getItemFromStack(current, 0)!}, immediate())
                        .to({opacity: 1, yPosition: stackStart - rectHeight * pushedPosition}),
                )
            }
            else if(operationKind === "Pop"){
                // One pop
                // [..., X] -> [...]
                const poppedPosition = hasExtra ? (maxShown-1) : (previous.length-1);
                
                all(
                    // Stack Items boxes
                    ...stackRectArray.map((rect,i) => {
                        const currVal = current[currentOffset+i];
                        if(currVal){
                            const endHeight = stackStart - rectHeight * i;
                            // If there is extra, boxes move up, else start height same as end height
                            const startHeight = hasExtra ? stackStart - rectHeight * Math.max(i-1, 0) : endHeight;
                            const startingOpacity = i === 0 && hasExtra ? 0 : 1;
                            // Move box up if room is made above.
                            // Fade in if from bottom
                            return rect
                                .to({opacity: startingOpacity, value: currVal, yPosition: startHeight}, immediate())
                                .to({opacity: 1, yPosition: endHeight});
                                // to({opacity:1, value: currVal, yPosition: stackStart - rectHeight*i}, immediate());
                        }
                        else{
                            // Item not on stack, so hide this box 
                            return rect.to({opacity:0}, immediate());
                        }
                    }),
                    // Rest of the Stack box
                    restOfStackRect.to({opacity: startingExtraOpacity}, immediate()).to({opacity: endingExtraOpacity}),
                    // Item being popped
                    topWorkingRect1
                        .to({opacity: 1, yPosition: stackStart - rectHeight * poppedPosition, value: getItemFromStack(previous, 0)!}, immediate())
                        .to({opacity: 0, yPosition: yOutside})
                )
            }
            else if(operationKind === "Unary"){
                console.log("UNARY")
                // One pop, apply op, then push
                // [..., X] -> (•) [X] -> [Y] -> [..., Y]
                // No check, assume there is at least one item

                // Step 1: "Pop" top item to the side
                const poppedPosition = previousHasExtra ? (maxShown-1) : (previous.length-1);
                
                all(
                    // Stack Items boxes
                    ...stackRectArray.map((rect,i) => {
                        const prevVal = previous[previousOffset+i-1];
                        if(prevVal && i < previous.length){
                            const endHeight = stackStart - rectHeight * i;
                            // If there is extra, boxes move up, else start height same as end height
                            const startHeight = previousHasExtra ? stackStart - rectHeight * Math.max(i-1, 0) : endHeight;
                            const startingOpacity = i === 0 && previousHasExtra ? 0 : 1;
                            // Move box up if room is made above.
                            // Fade in if from bottom
                            return rect
                                .to({opacity: startingOpacity, value: prevVal, yPosition: startHeight}, immediate())
                                .to({opacity: 1, yPosition: endHeight});
                                // to({opacity:1, value: currVal, yPosition: stackStart - rectHeight*i}, immediate());
                        }
                        else{
                            // Item not on stack, so hide this box 
                            return rect.to({opacity:0}, immediate());
                        }
                    }),
                    // Rest of the Stack box
                    restOfStackRect.to({opacity: startingExtraOpacity}, immediate()).to({opacity: endingExtraOpacity}),
                    // Item being popped
                    topWorkingRect2
                        .to({opacity: 1, yPosition: stackStart - rectHeight * poppedPosition, value: getItemFromStack(previous, 0)!}, immediate())
                        .to({opacity: 1, yPosition: yTopWorking, xPosition: xPosition+xWorkingOffset}),
                    operatorRect.to({opacity: 0, yPosition: yTopWorking + operatorSize/2, value:operation}, immediate())
                )
                // Step 2: Fade in operation
                .then(
                    async () => {
                        return await operatorRect.to({opacity:1})
                    }
                )
                // Step 3: Combine operation with item to produce new item
                .then(
                    async () => {
                        return all(
                            topWorkingRect2
                                .to({xPosition: xPosition})
                                .to({value: getItemFromStack(current, 0)}, immediate({delay:500})),
                            operatorRect.to({xPosition: xPosition, opacity: 0})
                        )
                    }
                )
                // Step 4: Push result to stack
                .then(
                    async () => {
                        const pushedPosition = currentHasExtra ? (maxShown-1) : (current.length-1);

                        return await all(
                            ...stackRectArray.map((rect,i) => {
                                const currVal = current[currentOffset+i-1];
                                if(currVal && i < current.length){
                                    const startHeight = stackStart - rectHeight * i;
                                    // No changes in height if there is no extra
                                    const endHeight = currentHasExtra ? stackStart - rectHeight * Math.max(i-1, 0) : startHeight;
                                    const endingOpacity = i === 0 && currentHasExtra ? 0 : 1;
                                    // Transform from previous value to current value and move box down
                                    return rect
                                        .to({opacity: 1, yPosition: startHeight, value: currVal}, immediate())
                                        // Fade out if at bottom
                                        .to({opacity: endingOpacity, yPosition: endHeight});
                                }
                                else{
                                    // Item not on stack, so hide this box 
                                    // - should unreachable, but kept just in case
                                    return rect.to({opacity:0}, immediate());
                                }
                            }),
                            // Rest of stack
                            restOfStackRect
                                .to({opacity: endingExtraOpacity}),
                            // Item being pushed onto the stack
                            topWorkingRect2
                                .to({yPosition: stackStart - rectHeight * pushedPosition}),
                        )
                    }
                )
                
            }
            else if(operationKind === "Binary"){
                // Two pop, apply op, then push
                // [..., X, Y] -> [X] (•) [Y] -> [Z] -> [..., Z]
                const poppedPositionTop = hasExtra ? (maxShown-1) : (previous.length-1);
                const poppedPositionBelow = hasExtra ? (maxShown-2) : (previous.length-2);

                // Step 1a: Pop top item to the side 1 (e.g., left)
                // Step 1b: Pop next top item to the side 2 (e.g. right)
                await all(
                    // Stack Items boxes
                    ...stackRectArray.map((rect,i) => {
                        const prevVal = previous[previousOffset+i-2];
                        if(prevVal && i < previous.length){
                            const endHeight = stackStart - rectHeight * i;
                            const midPointHeight = hasExtra ? stackStart - rectHeight * Math.max(i-1, 0) : endHeight;
                            // If there is extra, boxes move up, else start height same as end height
                            const startHeight = hasExtra ? stackStart - rectHeight * Math.max(i-2, 0) : endHeight;
                            const startingOpacity = i === 0 && hasExtra ? 0 : 1;
                            console.log("BINARY, I exist: ", previousOffset, i, previous)
                            // Move box up if room is made above.
                            // Fade in if from bottom
                            return rect
                                .to({opacity: startingOpacity, value: prevVal, yPosition: startHeight}, immediate())
                                .to({opacity: 1, yPosition: midPointHeight})
                                .to({yPosition: endHeight});
                        }
                        else{
                            // Item not on stack, so hide this box 
                            return rect.to({opacity:0}, immediate());
                        }
                    }),
                    // Rest of the Stack box
                    restOfStackRect.to({opacity: startingExtraOpacity}, immediate()).to({opacity: endingExtraOpacity}),
                    // Items being popped
                    topWorkingRect1
                        .to({opacity: 1, yPosition: stackStart - rectHeight * poppedPositionTop, value: getItemFromStack(previous, 0)!}, immediate())
                        .to({yPosition: yTopWorking, xPosition: xPosition+xWorkingOffset}),
                    topWorkingRect2
                        .to({opacity: 1, yPosition: stackStart - rectHeight * poppedPositionBelow, value: getItemFromStack(previous, 1)!}, immediate())
                        .to({yPosition: stackStart - rectHeight * poppedPositionTop})
                        .to({yPosition: yTopWorking, xPosition: xPosition-xWorkingOffset}),
                    operatorRect.to({opacity: 0, xPosition:xPosition+operatorSize, yPosition: yTopWorking + operatorSize/2, value:operation}, immediate())
                )
                // Step 2: Fade in operation (in-between)
                .then(
                    async () => {
                        return await operatorRect.to({opacity:1})
                    }
                )
                // Step 3: Combine operation with items to produce new item
                .then(
                    async () => {
                        return await all(
                            topWorkingRect1
                                .to({xPosition: xPosition, opacity:0}),
                            topWorkingRect2
                                .to({xPosition: xPosition})
                                .to({value: getItemFromStack(current, 0)}, immediate({delay:500})),
                            operatorRect.to({opacity: 0}, {delay:500})
                        )
                    }
                )
                // Step 4: Push result to stack
                .then(
                    async () => {
                        const pushedPosition = hasExtra ? (maxShown-1) : (current.length-1);

                        return await all(
                            ...stackRectArray.map((rect,i) => {
                                const currVal = current[currentOffset+i-1];
                                if(currVal && i < current.length){
                                    const startHeight = stackStart - rectHeight * i;
                                    // No changes in height if there is no extra
                                    const endHeight = hasExtra ? stackStart - rectHeight * Math.max(i-1, 0) : startHeight;
                                    const endingOpacity = i === 0 && hasExtra ? 0 : 1;
                                    // Transform from previous value to current value and move box down
                                    return rect
                                        .to({opacity: 1, yPosition: startHeight, value: currVal}, immediate())
                                        // Fade out if at bottom
                                        .to({opacity: endingOpacity, yPosition: endHeight});
                                }
                                else{
                                    // Item not on stack, so hide this box 
                                    // - should unreachable, but kept just in case
                                    return rect.to({opacity:0}, immediate());
                                }
                            }),
                            // Rest of stack
                            restOfStackRect
                                .to({opacity: endingExtraOpacity}),
                            // Item being pushed onto the stack
                            topWorkingRect2
                                .to({yPosition: stackStart - rectHeight * pushedPosition}),
                        )
                    }
                )
            }
        }
    } 

    return {buildAnimation, resetAnimation, signalData: {
        working1: topWorkingRect1,
        working2: topWorkingRect2,
        operator: operatorRect,
        restOfStack: restOfStackRect,
        stackArray: stackRectArray,
    }}
}