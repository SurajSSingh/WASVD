import { deserialize_number, instruction_in_plain_english } from "$lib";
import type * as command from "$lib/bindings"
import type { StackOperationKind } from "./stackAnim";

export type EvalResult = {
    instruction: command.SerializedInstruction,
    locals: VariableTableType,
    action: string,
    continuation: {label: string|number, goto: "Return"| "End" | "Else" | "Block"}|null
}

export type MyError = {
message: string  
};

type StackType = (bigint|number)[];

export type VariableTableType = {
    values: (number|bigint)[],
    mapping: {
        [key:string]: number
    }
};

export function getVariable(varTable: VariableTableType, name: string): bigint | number | undefined{
    return varTable.values[+name] ?? varTable.values[varTable.mapping[name]];
}

export type WasmData = {
    globals: VariableTableType,
    memory: {
        [key:string]: {[key:number]: string | number[]}
    },
    functions: command.WastFunc[],
}

function unimplemented_instruction_error(instruction: command.SerializedInstruction):MyError{
    return {message: `Unimplmented Instruction: ${instruction}`}
}

function unimplemented_error(msg: string):MyError{
    return {message: `Unimplmented Instruction: ${msg}`}
}

function not_enough_stack_error(expected: number, actual: number):MyError{
    return {message: `Not enough values on stack: expect at least ${expected}, but only got ${actual}`}
}

function stack_empty_error():MyError{
    return {message: "Stack is empty"}
}

function type_mismatch_error():MyError{
    return { message: "Types do not match"}
}

function data_not_found_error(location: string, type: "Global" | "Local" | "Memory"):MyError{
    return {message: `Data not found at ${location} (${type})`}
}

function name_resolution_error(label: string): MyError{
    return {message: `The label ${label} is not found in scope!`}
}

// function block_depth_exceeded_error(max_depth: number, actual: number):MyError{
//     return {message: `Exceeded block depth of ${max_depth}, but got ${actual}`}
// }

function unreachable_reached_error():MyError{
    return {message: "Reached an unreachable statement"}
}

function stack_pop(stack:StackType, amount: number): StackType|MyError{
    const subStack: StackType = []
    for (let i = 0; i < amount; i++) {
        const value = stack.pop();
        // console.log("STACK POP", value, subStack)
        if(value !== undefined){
            subStack.push(value)
        }
        else if (subStack.length === 0){
            return stack_empty_error();
        }
        else{
            // Put all values back in stack
            stack = stack.concat(subStack.reverse());
            return not_enough_stack_error(amount, subStack.length);
        }
    }
    return subStack;
}

function tryNumberify(location: string): number|string{
    const attemptedNumber = +location;
    return !isNaN(attemptedNumber) && attemptedNumber > -1 ? attemptedNumber : location ;
}

function math_operation(op: command.ArithmeticOperation | command.BitwiseOperation | command.ComparisonOperation | command.FloatOperation, first: number, second: number):number|MyError

function math_operation(op: command.ArithmeticOperation | command.BitwiseOperation | command.ComparisonOperation, first: bigint, second: bigint):bigint|MyError

function math_operation(op: command.ArithmeticOperation | command.BitwiseOperation | command.ComparisonOperation | command.FloatOperation, first: (bigint| number), second: (bigint| number|undefined)):(bigint| number|MyError)

function math_operation(op: command.ArithmeticOperation | command.BitwiseOperation | command.ComparisonOperation | command.FloatOperation, first: (bigint| number), second: (bigint| number|undefined)):(bigint| number|MyError){
    // Convert to 0|0n if second is not defined
    if(typeof second === 'undefined'){
        second = typeof first === "bigint" ? 0n : 0;
    }
    if(typeof first === "bigint" && typeof second === "bigint"){
        switch (op) {
            case "Addition":
                return first + second
            case "Subtraction":
                return first - second
            case "Multiplication":
                return first * second
            case "DivisonSigned":
                return first / second
            case "DivisonUnsigned":
                return first / second
            case "RemainderSigned":
                return first % second
            case "RemainderUnsigned":
                return first % second
            case "CountLeadingZero":{
                const notFirst = ~first;
                return Math.clz32(Number(notFirst & 0xFFFFFFFFn)) + Math.clz32(Number(notFirst >> 32n));
            }
            case "CountTrailingZero": {
                let count = 0;
                while ((first & 1n) === 0n && first !== 0n) {
                    first >>= 1n;
                    count++;
                }
                return count;
            } 
            case "CountNonZero":{
                let count = 0;
                while (first !== 0n) {
                    if ((first & 1n) === 1n) {
                        count++;
                    }
                    first >>= 1n;
                }
                return count;
            }
            case "And":
                return first & second
            case "Or":
                return first | second
            case "Xor":
                return first ^ second
            case "ShiftLeft":
                return first << second
            case "ShiftRightSigned":
                return first >> second
            case "ShiftRightUnsigned":
                // TODO: See if this is actually correct
                return first >> second
            case "RotateLeft": {
                const mask = (1n << second) - 1n;
                return ((first << second) | (first >> (32n - second))) & mask;
            }
            case "RotateRight":{
                const mask = (1n << second) - 1n;
                return ((first >> second) | (first << (32n - second))) & mask;
            }
            case "EqualZero":
                return first === 0n ? 1n : 0n;
            case "Equal":
                return first === second ? 1n : 0n;
            case "NotEqual":
                return first !== second ? 1n : 0n;
            case "LessThenSigned":
                return first < second ? 1n : 0n;
            case "LessThenUnsigned":
                return first < second ? 1n : 0n;
            case "GreaterThenSigned":
                return first > second ? 1n : 0n;
            case "GreaterThenUnsigned":
                return first > second ? 1n : 0n;
            case "LessThenOrEqualToSigned":
                return first <= second ? 1n : 0n;
            case "LessThenOrEqualToUnsigned":
                return first <= second ? 1n : 0n;
            case "GreaterThenOrEqualToSigned":
                return first >= second ? 1n : 0n;
            case "GreaterThenOrEqualToUnsigned":
                return first >= second ? 1n : 0n;
            case "AbsoluteValue":
            case "Negation":
            case "Ceiling":
            case "Floor":
            case "Truncate":
            case "Nearest":
            case "SquareRoot":
            case "Minimum":
            case "Maximum":
            case "CopySign":
            default:
                return unreachable_reached_error();
        }
    }
    else if(typeof first === "number" && typeof second === "number"){
        switch (op) {
            case "Addition":
                return first + second
            case "Subtraction":
                return first - second
            case "Multiplication":
                return first * second
            case "DivisonSigned":
                return first / second
            case "DivisonUnsigned":
                return Math.abs(first / second)
            case "RemainderSigned":
                return first % second
            case "RemainderUnsigned":
                return Math.abs(first % second)
                case "CountLeadingZero":
                    return Math.clz32(first);
                case "CountTrailingZero": {
                    let count = 0;
                    while ((first & 1) === 0 && first !== 0) {
                        first >>= 1;
                        count++;
                    }
                    return count;
                } 
                case "CountNonZero":{
                    let count = 0;
                    while (first !== 0) {
                        if ((first & 1) === 1) {
                            count++;
                        }
                        first >>= 1;
                    }
                    return count;
                }
            case "And":
                return first & second
            case "Or":
                return first | second
            case "Xor":
                return first ^ second
            case "ShiftLeft":
                return first << second
            case "ShiftRightSigned":
                return first >> second
            case "ShiftRightUnsigned":
                return first >>> second
            case "RotateLeft": {
                const mask = (1 << second) - 1;
                return ((first << second) | (first >>> (32 - second))) & mask;
            }
            case "RotateRight":{
                const mask = (1 << second) - 1;
                return ((first >>> second) | (first << (32 - second))) & mask;
            }
            case "EqualZero":
                return first === 0 ? 1 : 0;
            case "Equal":
                return first === second ? 1 : 0;
            case "NotEqual":
                return first !== second ? 1 : 0;
            case "LessThenSigned":
                return first < second ? 1 : 0;
            case "LessThenUnsigned":
                return first < second ? 1 : 0;
            case "GreaterThenSigned":
                return first > second ? 1 : 0;
            case "GreaterThenUnsigned":
                return first > second ? 1 : 0;
            case "LessThenOrEqualToSigned":
                return first <= second ? 1 : 0;
            case "LessThenOrEqualToUnsigned":
                return first <= second ? 1 : 0;
            case "GreaterThenOrEqualToSigned":
                return first >= second ? 1 : 0;
            case "GreaterThenOrEqualToUnsigned":
                return first >= second ? 1 : 0;
            case "AbsoluteValue":
                return Math.abs(first);
            case "Negation":
                return -first;
            case "Ceiling":
                return Math.ceil(first);
            case "Floor":
                return Math.floor(first);
            case "Truncate":
                return Math.trunc(first);
            case "Nearest":
                return Math.round(first);
            case "SquareRoot":
                return Math.sqrt(first);
            case "Minimum":
                return Math.min(first, second);
            case "Maximum":
                return Math.max(first, second);
            case "CopySign":
                return Math.sign(first) === Math.sign(second)? first: -first;
            default:
                return unreachable_reached_error();
        }
    }
    return type_mismatch_error();
}

export function formatStack(stack:(bigint|number)[], maxShown:number = 0):string{
    if(stack.length === 0){
        return "Empty"
    }
    else{
        let prefix = "";
        if(maxShown > 0){
            prefix = stack.length > maxShown ? "..., " : prefix;
            stack = stack.slice(-maxShown);
        }
        return `[${prefix}${stack.map(n => n.toString()).join(", ")}]`;
    }
}

export function eval_single_instruction(instruction: command.SerializedInstruction, stack: StackType, data: WasmData, locals: VariableTableType):EvalResult|MyError{
    if ("Simple" in instruction){
        switch (instruction.Simple) {
            case "Unreachable":
                return unreachable_reached_error();
            case "Nop":
                break;
            case "Return":
                return {instruction, action: instruction_in_plain_english(instruction), continuation: {label:0, goto:"Return"}, locals: structuredClone(locals)};
            case "Drop":
                stack.pop()
                break;
        }
    }else if("Block" in instruction){
        // All except If-else does nothing special (at least not until branch)
        if(instruction.Block.kind === "If"){
            // console.log("IF STARTING: ", stack);
            const numberOrError = stack_pop(stack, 1);
            // console.log("IF POPPED: ", numberOrError);
            if("message" in numberOrError){
                // is Error
                return numberOrError;
            }
            const [a] = numberOrError;
            // If zero, then skip to else
            if(a === 0 || a === 0n) {
                return {instruction, action: instruction_in_plain_english(instruction), continuation: {label: 0, goto:"Else"}, locals: structuredClone(locals)};
            }
            // Otherwise, continue until else
        }
        // If we reach an else, skip to end
        else if (instruction.Block.kind === "Else"){
            return {instruction, action: instruction_in_plain_english(instruction), continuation: {label: 0, goto:"End"}, locals: structuredClone(locals)};
        }
        
    }
    else if("Branch" in instruction){
        if(instruction.Branch.other_labels.length > 0){
            return unimplemented_instruction_error(instruction);
        }
        if(instruction.Branch.is_conditional){
            const numberOrError = stack_pop(stack, 1);
            if("message" in numberOrError){
                // is Error
                return numberOrError;
            }
            const [a] = numberOrError;
            // If zero, just go to next item
            if(a === 0 || a === 0n) {
                return {instruction, action: instruction_in_plain_english(instruction), continuation:null, locals: structuredClone(locals)};
            }
        }
        // Do branch to label or block index 
        return {instruction, action: instruction_in_plain_english(instruction), continuation: {label: tryNumberify(instruction.Branch.default_label), goto:"Block"}, locals: structuredClone(locals)}
    }
    else if("Call" in instruction){
        return unimplemented_instruction_error(instruction);
    }
    else if("Data" in instruction){
        switch (instruction.Data.kind) {
            case "GetLocal":{
                const local = getVariable(locals, instruction.Data.location);
                if(local !== undefined){
                    stack.push(local);
                }
                else{
                    return data_not_found_error(instruction.Data.location, "Local");
                }
                break;
            }
            case "GetGlobal":{
                const global = getVariable(data.globals, instruction.Data.location);
                if(global !== undefined){
                    stack.push(global);
                }
                else{
                    return data_not_found_error(instruction.Data.location, "Global");
                }
                break;
            }
            case "SetLocal":{
                const numberOrError = stack_pop(stack, 1);
                if("message" in numberOrError){
                    // is Error
                    return numberOrError;
                }
                const [a] = numberOrError;
                const index = tryNumberify(instruction.Data.location);
                if( typeof index === "number"){
                    locals.values[index] = a;
                }
                else {
                    locals.values[locals.mapping[index]] = a;
                }
                break;
            }
            case "SetGlobal":{
                const numberOrError = stack_pop(stack, 1);
                if("message" in numberOrError){
                    // is Error
                    return numberOrError;
                }
                const [a] = numberOrError;
                const index = tryNumberify(instruction.Data.location);
                if( typeof index === "number"){
                    data.globals.values[index] = a;
                }
                else {
                    data.globals.values[locals.mapping[index]] = a;
                }
                break;
            }
            case "TeeLocal":{
                const numberOrError = stack_pop(stack, 1);
                if("message" in numberOrError){
                    // is Error
                    return numberOrError;
                }
                const [a] = numberOrError;
                // Set
                const index = tryNumberify(instruction.Data.location);
                if( typeof index === "number"){
                    locals.values[index] = a;
                }
                else {
                    locals.values[locals.mapping[index]] = a;
                }
                // Get
                stack.push(getVariable(locals,instruction.Data.location) ?? a);
                break;
            }
            case "GetMemorySize":
                return unimplemented_instruction_error(instruction);
            case "SetMemorySize":
                return unimplemented_instruction_error(instruction);
            default:
                break;
        }
    }
    else if("Memory" in instruction){
        if(instruction.Memory.is_storing){
            return unimplemented_instruction_error(instruction);
        }
        else{
            return unimplemented_instruction_error(instruction);
        }
    }
    else if("Const" in instruction){
        stack.push(deserialize_number(instruction.Const.value));
    }
    else if("Comparison" in instruction){
        if(instruction.Comparison.kind === "EqualZero"){
            const numbers = stack_pop(stack, 1);
            if ("message" in numbers){
                // is Error
                return numbers;
            }
            const [a] = numbers;
            const result = math_operation(instruction.Comparison.kind, a, 0);
            if(typeof result !== 'object'){
                stack.push(result);
            }
            else{
                // is Error
                return result;
            }

        }
        else {
            const numbers = stack_pop(stack, 2);
            if ("message" in numbers){
                // is Error
                return numbers;
            }
            const [b,a] = numbers;
            const result = math_operation(instruction.Comparison.kind, a, b);
            if(typeof result !== 'object'){
                stack.push(result);
            }
            else{
                // is Error
                return result;
            }
        }
    }
    else if("Arithmetic" in instruction){
        const numbers = stack_pop(stack, 2);
        if ("message" in numbers){
            // is Error
            return numbers;
        }
        const [b,a] = numbers;
        const result = math_operation(instruction.Arithmetic.kind, a, b);
        if(typeof result !== 'object'){
            stack.push(result);
        }
        else{
            // is Error
            return result;
        }
    }
    else if("Bitwise" in instruction){
        switch (instruction.Bitwise.kind) {
            case "CountLeadingZero":
            case "CountTrailingZero":
            case "CountNonZero":{
                const numbers = stack_pop(stack, 1);
                if ("message" in numbers){
                    // is Error
                    return numbers;
                }
                const [a] = numbers;
                const result = math_operation(instruction.Bitwise.kind, a, instruction.Bitwise.is_64_bit ? 0n : 0);
                if(typeof result !== 'object'){
                    stack.push(result);
                }
                else{
                    // is Error
                    return result;
                }
                break;
            }
            case "And":
            case "Or":
            case "Xor":
            case "ShiftLeft":
            case "ShiftRightSigned":
            case "ShiftRightUnsigned":
            case "RotateLeft":
            case "RotateRight":{
                const numbers = stack_pop(stack, 2);
                if ("message" in numbers){
                    // is Error
                    return numbers;
                }
                const [b,a] = numbers;
                const result = math_operation(instruction.Bitwise.kind, a, b);
                if(typeof result !== 'object'){
                    stack.push(result);
                }
                else{
                    // is Error
                    return result;
                }
                break;
            }    
        }
    }
    else if("Float" in instruction){
        switch (instruction.Float.kind) {
            case "AbsoluteValue":
            case "Negation":
            case "Ceiling":
            case "Floor":
            case "Truncate":
            case "Nearest":
            case "SquareRoot":{
                const numbers = stack_pop(stack, 1);
                if ("message" in numbers){
                    // is Error
                    return numbers;
                }
                const [a] = numbers;
                const result = math_operation(instruction.Float.kind, a, 0);
                if(typeof result !== 'object'){
                    stack.push(result);
                }
                else{
                    // is Error
                    return result;
                }
                break;
            }
            case "Minimum":
            case "Maximum":
            case "CopySign":{
                const numbers = stack_pop(stack, 2);
                if ("message" in numbers){
                    // is Error
                    return numbers;
                }
                const [b,a] = numbers;
                const result = math_operation(instruction.Float.kind, a, b);
                if(typeof result !== 'object'){
                    stack.push(result);
                }
                else{
                    // is Error
                    return result;
                }
                break;
            }
        }
    }
    else if("Conversion" in instruction){
        const number = stack_pop(stack, 1);
        if("message" in number){
            // is Error
            return number;
        }
        const n = number[0];
        if(typeof n === "bigint"){
            switch(instruction.Conversion){
                case "WrapInt":{
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setBigInt64(0, n);
                    stack.push(view.getInt32(0));
                    break;
                }          
                case "SignedConvertI64ToF32":{
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setBigInt64(0, n);
                    stack.push(view.getFloat32(0));
                    break;
                }   
                case "UnsignedConvertI64ToF32":{
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setBigUint64(0, n);
                    stack.push(view.getFloat32(0));
                    break;
                }   
                case "SignedConvertI64ToF64":{
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setBigInt64(0, n);
                    stack.push(view.getFloat64(0));
                    break;
                }   
                case "UnsignedConvertI64ToF64":{
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setBigUint64(0, n);
                    stack.push(view.getFloat64(0));
                    break;
                }   
                case "Reinterpret64IToF":{
                    // 8 bytes for a 64-bit float
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setBigInt64(0, n);
                    stack.push(view.getFloat64(0));
                    break;
                }
                default:
                    return unreachable_reached_error()
            }
        }
        else {
            switch(instruction.Conversion){
                // number is 64-bit float, so certain conversions do nothing
                case "DemoteFloat":
                case "PromoteFloat":
                case "SignedConvertI32ToF32":
                case "SignedConvertI32ToF64":
                {
                    stack.push(n);
                    break;
                } 
                case "SignedTruncF32ToI32":
                case "SignedTruncF64ToI32":
                {
                    stack.push(Math.trunc(n));
                    break;
                } 
                case "UnsignedConvertI32ToF32":
                case "UnsignedConvertI32ToF64":
                {
                    stack.push(Math.abs(n));
                    break;
                } 
                case "UnsignedTruncF32ToI32":
                case "UnsignedTruncF64ToI32":
                {
                    stack.push(Math.abs(Math.trunc(n)));
                    break;
                } 
                case "SignedTruncF32ToI64":
                case "SignedTruncF64ToI64":
                case "SignedExtend":
                {
                    stack.push(BigInt(Math.trunc(n)));
                    break;
                } 
                case "UnsignedTruncF32ToI64":
                case "UnsignedTruncF64ToI64":
                case "UnsignedExtend":
                {
                    stack.push(BigInt(Math.abs(Math.trunc(n))));
                    break;
                } 
                case "Reinterpret32FToI": {
                    // 4 bytes for a 32-bit int
                    const buffer = new ArrayBuffer(4);
                    const view = new DataView(buffer);
                    view.setFloat32(0, n);
                    stack.push(view.getInt32(0));
                    break;
                }
                case "Reinterpret32IToF":{
                    // 4 bytes for a 32-bit float
                    const buffer = new ArrayBuffer(4);
                    const view = new DataView(buffer);
                    view.setInt32(0, n);
                    stack.push(view.getFloat32(0));
                    break;
                }
                case "Reinterpret64FToI":{
                    // 8 bytes for a 64-bit int
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setFloat64(0, n);
                    stack.push(view.getBigInt64(0));
                    break;
                }
                    
                default:
                    return unreachable_reached_error();
            }
        }
    }
    return {instruction, action: instruction_in_plain_english(instruction), continuation: null, locals: structuredClone(locals)}
}

export function exec_instructions(tree: command.SerializedInstructionTree, data: WasmData, locals:VariableTableType):{result: EvalResult; previous: (number | bigint)[]; current: (number | bigint)[];}[]|MyError{
    const currentStack: (bigint|number)[] = [];
    let previousStack;
    const final: {
        result: EvalResult;
        previous: (number | bigint)[];
        current: (number | bigint)[];
    }[] = [];
    // console.log(JSON.stringify(tree.root));
    let current_step = 0;
    let currentBlock = tree.root[0];
    for (let index = currentBlock.start; index < tree.array.length;) {
        // console.log(`BEFORE ${index}: `, [currentBlock.kind, currentBlock.label, currentBlock.depth], [currentBlock.start, currentBlock.end], [currentBlock.parent, currentBlock.children]);
        // Update current block first
        if(index === currentBlock.end){
            // Index at end of block, so go back up to parent
            currentBlock = tree.root[currentBlock.parent];
            // console.log("CHILD -> PARENT");
        }
        if(index in currentBlock.children){
            // Index in children, so get change block to child
            const childIndex = currentBlock.children[index];
            currentBlock = tree.root[childIndex];
            // console.log("PARENT -> CHILD");
        }
        // Evaluate instruction
        const instruction = tree.array[index];
        previousStack = structuredClone(currentStack);
        const action = eval_single_instruction(instruction, currentStack, data, locals);
        // Return errors immediately
        if("message" in action){
            // console.log(final);
            action.message += `(@ Step ${current_step} on ${instruction_in_plain_english(instruction)})`
            return action;
        }
        // Push result
        final.push({
            result: action,
            previous: structuredClone(previousStack),
            current: structuredClone(currentStack),
        })
        // If continuation is not null, choose where to go next
        if(action.continuation !== null){
            // console.log("Branch happened: ", action.action, action.continuation)
            if(action.continuation.goto === "Return"){
                // console.log("RETURN IMMEDIATELY");
                break;
            }
            else if(action.continuation.goto === "Else" && typeof currentBlock.kind === "object"){
                // console.log(`Skip to ELSE: from ${index} to ${currentBlock.kind.Conditional}`);
                // Go to where the else starts (skipping then)
                index = currentBlock.kind.Conditional;
            }
            else if(action.continuation.goto === "End" && typeof currentBlock.kind === "object"){
                // console.log(`Skip to END: from ${index} to ${currentBlock.end}`);
                // Go to the end of conditional (skipping else)
                index = currentBlock.end;
            }
            else if ((action.continuation.goto === "Else" || action.continuation.goto === "End") && typeof currentBlock.kind !== "object"){
                return unimplemented_error("Else or End can only be continued from conditional");
            }
            else if(action.continuation.goto === "Block"){
                // Number label -> go to nth enclosing block, current block is 0
                if(typeof action.continuation.label === "number" ){
                    while(action.continuation.label > 0){
                        // console.log(`${action.continuation.label} CHILD -> PARENT (${currentBlock.parent})`);
                        currentBlock = tree.root[currentBlock.parent];
                        action.continuation.label--;
                    }
                    if (currentBlock.kind === "Block"){
                        // console.log(`Skip to END: from ${index} to ${currentBlock.end}`);
                        // For block, go to end
                        index = currentBlock.end;
                    }
                    else if (currentBlock.kind === "Loop"){
                        // console.log(`Skip to START: from ${index} to ${currentBlock.start}`);
                        // For loop, go to start
                        index = currentBlock.start
                    }
                }
                // String label -> go to enclosing block with the label, traversing up parents
                else if(typeof action.continuation.label === "string"){
                    while(action.continuation.label !== currentBlock.label && currentBlock.depth !== 0){
                        // console.log(`${action.continuation.label} CHILD -> PARENT (${currentBlock.parent})`);
                        currentBlock = tree.root[currentBlock.parent];
                    }
                    // Return error if label is not found
                    if(action.continuation.label !== currentBlock.label){
                        return name_resolution_error(action.continuation.label);
                    }
                    // Otherwise, update index
                    if (currentBlock.kind === "Block"){
                        // console.log(`Skip to END: from ${index} to ${currentBlock.end}`);
                        // For block, go to end
                        index = currentBlock.end;
                    }
                    else if (currentBlock.kind === "Loop"){
                        // console.log(`Skip to START: from ${index} to ${currentBlock.start}`);
                        // For loop, go to start
                        index = currentBlock.start
                    }

                }
            }
        }
        // console.log(instruction_in_plain_english(tree.array[index]));
        index++;
        current_step++;
        // console.log(`AFTER ${index-1}: `, [currentBlock.kind, currentBlock.label, currentBlock.depth], [currentBlock.start, currentBlock.end], [currentBlock.parent, currentBlock.children]);
    }

    return final;
}

export function operationFromInstruction(instruction: command.SerializedInstruction): {kind: StackOperationKind, name: string}{
    if ("Simple" in instruction){
        switch (instruction.Simple) {
            case "Unreachable":
                return {kind: "Nop", name: "Unreachable"};
            case "Nop":
                return {kind: "Nop", name: "Nop"};
            case "Return":
                return {kind: "Nop", name: "Return"};
            case "Drop":
                return {kind: "Pop", name: "Drop"};
        }
    }else if("Block" in instruction){
        // If pops a value to check, everyone else just continues from stack
        // TODO: Add a way to "hide" outer scope stack items, since they can't be used
        if(instruction.Block.kind === "If"){
            return {kind: "Pop", name: instruction.Block.kind}
        }
        return {kind: "Nop", name: instruction.Block.kind}
    }
    else if("Branch" in instruction){
        if(instruction.Branch.other_labels.length > 0 || instruction.Branch.is_conditional){
            return {kind: "Pop", name: instruction.Branch.other_labels.length > 0 ? "Branch Table" : "Branch If"}
        }
        return {kind: "Nop", name: "Unconditional Branch"}
    }
    else if("Call" in instruction){
        // TODO: Need to make stack change on call
    }
    else if("Data" in instruction){
        switch (instruction.Data.kind) {
            case "GetLocal":
            case "GetGlobal":
                return {kind: "Push", name: instruction.Data.kind}
            case "SetLocal":
            case "SetGlobal":
                return {kind: "Pop", name: instruction.Data.kind}
            case "TeeLocal":
                return {kind: "Unary", name: "Tee"}
            case "GetMemorySize":
                return {kind: "Push", name: "Get Memory"}
            case "SetMemorySize":
                return {kind: "Unary", name: "Mem Resize"}
            default:
                break;
        }
    }
    else if("Memory" in instruction){
        // Skipped for now
    }
    else if("Const" in instruction){
        return {kind: "Push", name: "Const"}
    }
    else if("Comparison" in instruction){
        switch(instruction.Comparison.kind){
            case "EqualZero":
                return {kind: "Unary", name: instruction.Comparison.kind}
            case "Equal":
            case "NotEqual":
            case "LessThenSigned":
            case "LessThenUnsigned":
            case "GreaterThenSigned":
            case "GreaterThenUnsigned":
            case "LessThenOrEqualToSigned":
            case "LessThenOrEqualToUnsigned":
            case "GreaterThenOrEqualToSigned":
            case "GreaterThenOrEqualToUnsigned":
                return {kind: "Binary", name: instruction.Comparison.kind}
        }
    }
    else if("Arithmetic" in instruction){
        return {kind: "Binary", name: instruction.Arithmetic.kind}
    }
    else if("Bitwise" in instruction){
        switch (instruction.Bitwise.kind) {
            case "CountLeadingZero":
            case "CountTrailingZero":
            case "CountNonZero":{
                return {kind: "Unary", name: instruction.Bitwise.kind};
            }
            case "And":
            case "Or":
            case "Xor":
            case "ShiftLeft":
            case "ShiftRightSigned":
            case "ShiftRightUnsigned":
            case "RotateLeft":
            case "RotateRight":{
                return {kind: "Binary", name: instruction.Bitwise.kind};
            }    
        }
    }
    else if("Float" in instruction){
        switch (instruction.Float.kind) {
            case "AbsoluteValue":
            case "Negation":
            case "Ceiling":
            case "Floor":
            case "Truncate":
            case "Nearest":
            case "SquareRoot":{
                return {kind: "Unary", name: instruction.Float.kind}; 
            }
            case "Minimum":
            case "Maximum":
            case "CopySign":{
                return {kind: "Binary", name: instruction.Float.kind};
            }
        }
    }
    else if("Conversion" in instruction){
        return {kind: "Unary", name: instruction.Conversion}
    }
    return {kind: "Nop", name: "Unknown"}
}