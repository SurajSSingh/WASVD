import { instruction_in_plain_english } from "$lib";
import type * as command from "$lib/bindings"

export type EvalResult = {
    action: string,
    continuation: string|number|null
}

export type MyError = {
message: string  
};

type StackType = (bigint|number)[];

function unimplemented_error(instruction: command.SerializedInstruction):MyError{
    return {message: `Unimplmented Instruction: ${instruction}`}
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

function unreachable_reached_error():MyError{
    return {message: "Reached an unreachable statement"}
}

// function unknown_instruction_error(instruction: command.SerializedInstruction):Error{
//     return {message: `Reached an unknown instruction: ${instruction}`}
// }

function stack_pop(stack:StackType, amount: number): StackType|MyError{
    const subStack: StackType = []
    for (let i = 0; i < amount; i++) {
        const value = stack.pop();
        if(value){
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


export function eval_single_instruction(instruction: command.SerializedInstruction, stack: StackType):EvalResult|MyError{
    if ("Simple" in instruction){
        switch (instruction.Simple) {
            case "Unreachable":
                return unreachable_reached_error();
            case "Nop":
                break;
            case "Return":
                return {action: instruction_in_plain_english(instruction), continuation: null};
            case "Drop":
                stack.pop()
                break;
        }
    }else if("Block" in instruction){
        return unreachable_reached_error();
    }
    else if("Branch" in instruction){
        if(instruction.Branch.other_labels){
            return unimplemented_error(instruction);
        }
        return unimplemented_error(instruction);
    }
    else if("Call" in instruction){
        return unimplemented_error(instruction);
    }
    else if("Data" in instruction){
        switch (instruction.Data.kind) {
            case "GetLocal":
                return unimplemented_error(instruction);
            case "GetGlobal":
                return unimplemented_error(instruction);
            case "SetLocal":
                return unimplemented_error(instruction);
            case "SetGlobal":
                return unimplemented_error(instruction);
            case "TeeLocal":
                return unimplemented_error(instruction);
            case "GetMemorySize":
                return unimplemented_error(instruction);
            case "SetMemorySize":
                return unimplemented_error(instruction);
            default:
                break;
        }
    }
    else if("Memory" in instruction){
        if(instruction.Memory.is_storing){
            return unimplemented_error(instruction);
        }
        else{
            return unimplemented_error(instruction);
        }
    }
    else if("Const" in instruction){
        stack.push(deserialize_number(instruction.Const.value));
    }
    else if("Comparison" in instruction){
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
    else if("Cast" in instruction){
        const number = stack_pop(stack, 1);
        if("message" in number){
            // is Error
            return number;
        }
        const n = number[0];
        if(typeof n === "bigint"){
            switch(instruction.Cast){
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
            switch(instruction.Cast){
                case "SignedTruncF32ToI32":{
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setFloat32(0, n);
                    stack.push(view.getInt32(0));
                    break;
                } 
                case "UnsignedTruncF32ToI32":{
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setFloat32(0, n);
                    stack.push(view.getUint32(0));
                    break;
                } 
                case "SignedTruncF32ToI64":{
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setFloat32(0, n);
                    stack.push(view.getBigInt64(0));
                    break;
                } 
                case "UnsignedTruncF32ToI64":{
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setFloat32(0, n);
                    stack.push(view.getUint32(0));
                    break;
                } 
                case "SignedTruncF64ToI64":{
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setFloat64(0, n);
                    stack.push(view.getInt32(0));
                    break;
                } 
                case "UnsignedTruncF64ToI64":{
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setFloat64(0, n);
                    stack.push(view.getUint32(0));
                    break;
                } 
                case "SignedTruncF64ToI32":{
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setFloat64(0, n);
                    stack.push(view.getBigInt64(0));
                    break;
                } 
                case "UnsignedTruncF64ToI32":{
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setFloat64(0, n);
                    stack.push(view.getUint32(0));
                    break;
                } 
                case "SignedExtend":{
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setInt32(0, n);
                    stack.push(view.getBigInt64(0));
                    break;
                } 
                case "UnsignedExtend":{
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setUint32(0, n);
                    stack.push(view.getBigUint64(0));
                    break;
                } 
                case "SignedConvertI32ToF32":{
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setInt32(0, n);
                    stack.push(view.getFloat32(0));
                    break;
                } 
                case "UnsignedConvertI32ToF32":{
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setUint32(0, n);
                    stack.push(view.getFloat32(0));
                    break;
                } 
                case "SignedConvertI32ToF64":{
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setInt32(0, n);
                    stack.push(view.getFloat64(0));
                    break;
                } 
                case "UnsignedConvertI32ToF64":{
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setUint32(0, n);
                    stack.push(view.getFloat64(0));
                    break;
                } 
                case "DemoteFloat": {
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setFloat64(0, n);
                    stack.push(view.getFloat32(0));
                    break;
                }
                case "PromoteFloat": {
                    const buffer = new ArrayBuffer(8);
                    const view = new DataView(buffer);
                    view.setFloat32(0, n);
                    stack.push(view.getFloat64(0));
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
    return {action: instruction_in_plain_english(instruction), continuation: ""}
}

export function* eval_instruction_node(node: command.SerializedInstructionNode, stack: (bigint|number)[]):Generator<EvalResult|MyError, null|MyError> {
    let yieldableNodes;
    if("NonBlock" in node){
        yield eval_single_instruction(node.NonBlock, stack)
    } 
    else if ("SingleBlock" in node) {
        yieldableNodes = node.SingleBlock.inner_nodes;
        // for (const instruction of node.SingleBlock.inner_nodes) {
        //     yield* eval_instruction_node(instruction, stack)
        // }
    }
    else if ("ConditionalBlock" in node) {
        const popped = stack_pop(stack, 1);
        if("message" in popped){
            return stack_empty_error()
        }
        const [conditionalValue] = popped;
        yieldableNodes = (typeof conditionalValue === "number" && conditionalValue !== 0) 
                      || (typeof conditionalValue === "bigint" && conditionalValue !== 0n) 
                      ? node.ConditionalBlock.then_nodes
                      : node.ConditionalBlock.else_nodes;
        // if((typeof conditionalValue === "number" && conditionalValue !== 0) 
        // || (typeof conditionalValue === "bigint" && conditionalValue !== BigInt(0))){
        //     for (const instruction of node.ConditionalBlock.then_nodes) {
        //         yield* eval_instruction_node(instruction, stack)
        //     }
        // }
        // else {
        //     for (const instruction of node.ConditionalBlock.else_nodes) {
        //         yield* eval_instruction_node(instruction, stack)
        //     }
        // }
    }
    if(yieldableNodes){
        for (const instruction of yieldableNodes) 
        {
            yield* eval_instruction_node(instruction, stack);
        }
    }
    return null;
}


export function exec_instructions(tree: command.SerializedInstructionTree):{result: EvalResult; previous: (number | bigint)[]; current: (number | bigint)[];}[]|MyError{
    const stack: (bigint|number)[] = [];
    const final: {
        result: EvalResult;
        previous: (number | bigint)[];
        current: (number | bigint)[];
    }[] = [];
    for (const node of tree.root) {
        // console.log(JSON.stringify(node))
        let previousStack = structuredClone(stack);
        topLoop: for (const action of eval_instruction_node(node,stack)){
            if("message" in action){
                return action;
            }
            final.push({result: action, previous:previousStack, current:structuredClone(stack)})
            previousStack = structuredClone(stack);
            if(action.continuation === null){
                // Null continuation means immediate return
                break topLoop;
            }
            // console.log("RESULT: ", result);
        }
    }
    return final;
}
