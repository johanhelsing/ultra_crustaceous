import init from "./pkg/wasm_by_example_hello.js"

const runWasm = async () => {
    const mod = await WebAssembly.compileStreaming(fetch("./pkg/wasm_by_example_hello_bg.wasm"));
    console.log(window.a = mod)

    const rustWasm = await init("./pkg/wasm_by_example_hello_bg.wasm")
    rustWasm.store_value_in_wasm_memory_buffer_index_zero(24);

    let wasmMemory = new Uint8Array(rustWasm.memory.buffer);


    let bufferPointer = rustWasm.get_wasm_memory_buffer_pointer();

    console.log(wasmMemory[bufferPointer + 0]);

    console.log("Write in js read in wasm, index 1");

    wasmMemory[bufferPointer + 1] = 15;

    console.log(rustWasm.read_wasm_memory_buffer_and_return_index_one());
}

runWasm()
