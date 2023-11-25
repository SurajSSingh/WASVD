// place files you want to import through the `$lib` alias in this folder.
import type * as command from "$lib/bindings"

export function deserialize_number(serNumber: command.SerializedNumber): number
export function deserialize_number(serNumber: command.SerializedNumber & ({typ:"I64"} | {typ:"V128"})): bigint
export function deserialize_number(serNumber: command.SerializedNumber): bigint | number{
    const bytes = serNumber.second_bytes ? serNumber.first_bytes.concat(serNumber.second_bytes) :serNumber.first_bytes;
    const buffer = new ArrayBuffer(8);
    const view = new DataView(buffer);
    for (let i = 0; i < bytes.length; i++) {
        view.setUint8(i, bytes[i]);
     }
    if (serNumber.typ === "I64"){
        return view.getBigInt64(0)
    }
    else if (serNumber.typ !== "V128"){
        switch (serNumber.typ){
            case "I32":
                return view.getInt32(0)
                case "F32":
                return view.getFloat32(0)
                case "F64":
                return view.getFloat64(0)
        }
    }
    return 0
}

function formatInOut(inout: command.InputOutput | null, prefix: string = ": "){
    if(inout){
        const index = inout.index ? `Index(${inout.index}), ` : "";
        const params = inout.input.flatMap((x,i) => `${x[0] ?? i.toString()} ${x[1]}`).join(", ");
        const results = inout.output.join(", ");
        return `${prefix}${index}Params(${params}), Results(${results})`
    }
    return ""
}

export function instruction_in_plain_english(instruction: command.SerializedInstruction):string {

    if ("Simple" in instruction){
        switch (instruction.Simple) {
            case "Unreachable":
                return "Unreachable";
            case "Nop":
                return "Do nothing";
            case "Return":
                return "Return immediately";
            case "Drop":
                return "Drop top value from stack"
            default:
                return `UNKNOWN SIMPLE: ${instruction.Simple}`
        }
    }else if("Block" in instruction){
        switch (instruction.Block.kind) {
            case "Block":
                return `Start New Block ${instruction.Block.label}${formatInOut(instruction.Block.inout)}`
            case "If":
                return `Start If block ${instruction.Block.label}${formatInOut(instruction.Block.inout)}`
            case "Loop":
                return `Start Loop block ${instruction.Block.label}${formatInOut(instruction.Block.inout)}`
            case "Else":
                return `Start Else block ${instruction.Block.label}`
            case "End":
                return `End current block ${instruction.Block.label}`
        }
        return `Start ${instruction.Block.kind} ${instruction.Block.label}: ${JSON.stringify(instruction.Block.inout)}`;
    }
    else if("Branch" in instruction){
        if(instruction.Branch.other_labels){
            return `Branch Table: Cases: ${instruction.Branch.other_labels}, default: ${instruction.Branch.default_label}.`;
        }
        return `Branch ${instruction.Branch.is_conditional ? "if value on stack is 0" : "unconditionally"} to ${instruction.Branch.default_label}.`
    }
    else if("Call" in instruction){
        return `Call ${instruction.Call.index} with ${instruction.Call.inout.input || "no input"} from stack, which put back on stack: ${instruction.Call.inout.output || "nothing"}.`
    }
    else if("Data" in instruction){
        switch (instruction.Data.kind) {
            case "GetLocal":
                return `Get value from Local variable $${instruction.Data.location} and push it onto stack.`;
            case "GetGlobal":
                return `Get value from Global variable $${instruction.Data.location} and push it onto stack.`;
            case "SetLocal":
                return `Pop value from stack and set Local variable $${instruction.Data.location} to it.`;
            case "SetGlobal":
                return `Pop value from stack and set Global variable $${instruction.Data.location} to it.`;
            case "TeeLocal":
                return `Pop value from stack, set Local variable $${instruction.Data.location} to it, then push value back onto stack.`;
            case "GetMemorySize":
                return `Push size of memory location $${instruction.Data.location} onto stack.`;
            case "SetMemorySize":
                return `Pop value from stack, attempt to grow memory location $${instruction.Data.location} by it, and push old memory size if successful or else -1 to stack.`;
            default:
                break;
        }
    }
    else if("Memory" in instruction){
        if(instruction.Memory.is_storing){
            return `Storing ${instruction.Memory.count} of ${instruction.Memory.typ} to offset ${instruction.Memory.offset} (alignment: ${instruction.Memory.alignment}) at ${instruction.Memory.location}.`
        }
        else{
            return `Loading from ${instruction.Memory.location} at offset ${instruction.Memory.offset} (alignment: ${instruction.Memory.alignment}) ${instruction.Memory.count} of type ${instruction.Memory.typ}.`
        }
    }
    else if("Const" in instruction){
        return `Push constant ${deserialize_number(instruction.Const.value)} of type ${instruction.Const.typ} to stack.`
    }
    else if("Comparison" in instruction){
        let signedness = '';
        let message = '';
        switch (instruction.Comparison.kind){
            case "EqualZero":
                return `Pop ${instruction.Comparison.typ} value from stack, push 1 if value equals 0 otherwise 0.`
            case "Equal":
                message = "first value equals second";
                break;
                case "NotEqual":
                message = "first value does not equal second";
                break;
                case "LessThenSigned":
                signedness="signed"
                message = "first value less than second";
                break;
                case "LessThenUnsigned":
                signedness="unsigned"
                message = "first value less than second";
                break;
                case "GreaterThenSigned":
                    signedness="signed"
                message = "first value is greater than second";
                break;
                case "GreaterThenUnsigned":
                    signedness="unsigned"
                message = "first value is greater than second";
                break;
                case "LessThenOrEqualToSigned":
                    signedness="signed"
                message = "first value is less than or equals second";
                break;
                case "LessThenOrEqualToUnsigned":
                    signedness="unsigned"
                message = "first value is less than or equals second";
                break;
                case "GreaterThenOrEqualToSigned":
                    signedness="signed"
                message = "first value is greater than or equals second";
                break;
                case "GreaterThenOrEqualToUnsigned":
                    signedness="unsigned"
                message = "first value is greater than or equals second";
                break;
            }
            return `Pop top 2 ${signedness} ${instruction.Comparison.typ} values from stack, push 1 if ${message} otherwise push 0.`;
    }
    else if("Arithmetic" in instruction){
        let signedness = '';
        let operation = '';
        switch(instruction.Arithmetic.kind){
            case "Addition":
                operation='first + second'
                break;
            case "Subtraction":
                operation='first - second'
                break;
            case "Multiplication":
                operation='first * second'
                break;
            case "DivisonSigned":
                signedness='signed'
                operation='first / second'
                break;
            case "DivisonUnsigned":
                signedness='unsigned'
                operation='first / second'
                break;
            case "RemainderSigned":
                signedness='signed'
                operation='first % second'
                break;
            case "RemainderUnsigned":
                signedness='unsigned'
                operation='first % second'
                break;
        }
        return `Pop top 2 ${signedness} ${instruction.Arithmetic.typ} values from stack, push ${operation} result to stack.`
    }
    else if("Bitwise" in instruction){
        let number_to_pop = 0;
        let operation = "";
        switch (instruction.Bitwise.kind) {
            case "CountLeadingZero":
                number_to_pop = 1
                operation = "number of leading zero bits of value"
                break;
            case "CountTrailingZero":
                number_to_pop = 1
                operation = "number of trailing zero bits of value"
                break;
            case "CountNonZero":
                number_to_pop = 1
                operation = "number of non-zero bits of value"
                break;
            case "And":
                number_to_pop = 2
                operation = "bitwise and result of two values"
                break;
            case "Or":
                number_to_pop = 2
                operation = "bitwise or result of two values"
                break;
            case "Xor":
                number_to_pop = 2
                operation = "bitwise xor result of two values"
                break;
            case "ShiftLeft":
                number_to_pop = 2
                operation = "left shift first by second value result"
                break;
            case "ShiftRightSigned":
                number_to_pop = 2
                operation = "sign preserving right shift first by second values result"
                break;
            case "ShiftRightUnsigned":
                number_to_pop = 2
                operation = "sign ignoring right shift first by second values result"
                break;
            case "RotateLeft":
                number_to_pop = 2
                operation = "left bit rotation of first by second values result"
                break;
            case "RotateRight":
                number_to_pop = 2
                operation = "right bit rotation of first by second values result"
                break;
        }
        return `Pop ${number_to_pop} ${instruction.Bitwise.is_64_bit ? "I64" : "I32"} ${number_to_pop === 1? "value" : "values"} from stack, push ${operation} to stack.`
    }
    else if("Float" in instruction){
        let number_to_pop = 0;
        let operation = "";
        switch (instruction.Float.kind) {
            case "AbsoluteValue":
                number_to_pop = 1
                operation = "absolute value of value"
                break;
            case "Negation":
                number_to_pop = 1
                operation = "negation of value"
                break;
            case "Ceiling":
                number_to_pop = 1
                operation = "ceiling of value"
                break;
            case "Floor":
                number_to_pop = 1
                operation = "floor of value"
                break;
            case "Truncate":
                number_to_pop = 1
                operation = "truncation towards 0 of value"
                break;
            case "Nearest":
                number_to_pop = 1
                operation = "nearest even integer of value"
                break;
            case "SquareRoot":
                number_to_pop = 1
                operation = "square root of value"
                break;
            case "Minimum":
                number_to_pop = 2
                operation = "minimum of the two values"
                break;
            case "Maximum":
                number_to_pop = 2
                operation = "maximum of the two values"
                break;
            case "CopySign":
                number_to_pop = 2
                operation = "first value unchanged if values share the same sign else negation of first value"
                break;
        }
        return `Pop top ${number_to_pop} ${instruction.Float.is_64_bit ? "I64" : "I32"} ${number_to_pop === 1? "value" : "values"} from stack, push ${operation} to stack.`
    }
    else if("Cast" in instruction){
        switch(instruction.Cast){
            case "WrapInt":
                return "Wrap I64 to I32."
            case "SignedTruncF32ToI32":
                return "Truncate an F32 to a signed I32."
            case "UnsignedTruncF32ToI32":
                return "Truncate an F32 to an unsigned I32."
            case "SignedTruncF64ToI32":
                return "Truncate an F64 to a signed I32."
            case "UnsignedTruncF64ToI32":
                return "Truncate an F64 to an unsigned I32."
            case "SignedTruncF32ToI64":
                return "Truncate an F32 to a signed I64."
            case "UnsignedTruncF32ToI64":
                return "Truncate an F32 to an unsigned I64."
            case "SignedTruncF64ToI64":
                return "Truncate an F64 to a signed I64."
            case "UnsignedTruncF64ToI64":
                return "Truncate an F64 to an unsigned I64."
            case "SignedExtend":
                return "Extend an I32 to a signed I64."
            case "UnsignedExtend":
                return "Extend an I32 to an unsigned I64."
            case "SignedConvertI32ToF32":
                return "Convert an I32 to a signed F32."
            case "UnsignedConvertI32ToF32":
                return "Convert an I32 to an unsigned F32."
            case "SignedConvertI64ToF32":
                return "Convert an I64 to a signed F32."
            case "UnsignedConvertI64ToF32":
                return "Convert an I64 to an unsigned F32."
            case "SignedConvertI32ToF64":
                return "Convert an I32 to a signed F64."
            case "UnsignedConvertI32ToF64":
                return "Convert an I32 to an unsigned F64."
            case "SignedConvertI64ToF64":
                return "Convert an I64 to a signed F64."
            case "UnsignedConvertI64ToF64":
                return "Convert an I64 to an unsigned F64."
            case "DemoteFloat":
                return "Demote an F64 to an F32."
            case "PromoteFloat":
                return "Promote an F32 to an F64."
            case "Reinterpret32FToI":
                return "Reinterpret a 32-bit value from float to int."
            case "Reinterpret32IToF":
                return "Reinterpret a 32-bit value from int to float."
            case "Reinterpret64FToI":
                return "Reinterpret a 64-bit value from float to int."
            case "Reinterpret64IToF":
                return "Reinterpret a 64-bit value from int to float."
        }
    }
    return JSON.stringify(instruction);
}
