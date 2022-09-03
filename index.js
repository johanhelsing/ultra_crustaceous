const loadModule = async (path) => {
    const response = await fetch(path)
    const imports = {}
    const { instance } = await WebAssembly.instantiateStreaming(response, imports)
    return instance.exports
}

const inputBits = {
    up: 1 << 0,
    down: 1 << 1,
    left: 1 << 2,
    right: 1 << 3,
    button1: 1 << 4,
    button2: 1 << 5,
}

const startGame = async (canvas, game) => {
    const ctx = canvas.getContext('2d');
    canvas.width = 320;
    canvas.height = 240;

    // track key presses
    var pressedKeys = {};
    window.addEventListener("keyup", e => pressedKeys[e.code] = false)
    window.addEventListener("keydown", e => pressedKeys[e.code] = true)

    const getP1Input = () => 0
        | pressedKeys["ArrowUp"] ? inputBits.up : 0
        | pressedKeys["ArrowDown"] ? inputBits.down : 0
        | pressedKeys["ArrowLeft"] ? inputBits.left : 0
        | pressedKeys["ArrowRight"] ? inputBits.right : 0
        | pressedKeys["ShiftRight"] ? inputBits.button1 : 0
        | pressedKeys["Space"] || pressedKeys["Enter"] ? inputBits.button2 : 0

    const getP2Input = () => 0
        | pressedKeys["KeyW"] ? inputBits.up : 0
        | pressedKeys["KeyS"] ? inputBits.down : 0
        | pressedKeys["KeyA"] ? inputBits.left : 0
        | pressedKeys["KeyD"] ? inputBits.right : 0
        | pressedKeys["CtrlLeft"] || pressedKeys["KeyZ"] ? inputBits.button1 : 0
        | pressedKeys["ShiftLeft"] || pressedKeys["KeyX"] ? inputBits.button2 : 0

    const imageData = ctx.createImageData(320, 240);

    const imageBuffer = imageData.data;
    for (let i = 0; i < imageBuffer.length; i += 4) {
        imageBuffer[i + 3] = 255; // full alpha
    }

    const palette = [];

    const update = () => {
        const p1 = getP1Input();
        const p2 = getP2Input();
        game.update(p1, p2);

        const outputPointer = game.get_screen_buffer_pointer();
        const palettePointer = game.get_palette_buffer_pointer();

        const wasmByteMemoryArray = new Uint8Array(game.memory.buffer)
        const screenBufferArray = wasmByteMemoryArray.slice(
            outputPointer,
            outputPointer + 320 * 240
        );

        const paletteBufferArray = wasmByteMemoryArray.slice(
            palettePointer,
            palettePointer + 32 * 2
        );

        for (let i = 0; i < 32; ++i) {
            const r = paletteBufferArray[i * 2] << 4;
            const gb = paletteBufferArray[i * 2 + 1];
            const g = gb & 0b111100000;
            const b = (gb & 0b1111) << 4;
            palette[i] = { r, g, b }
        }

        for (let i = 0; i < screenBufferArray.length; ++i) {
            const paletteIndex = screenBufferArray[i];
            const color = palette[paletteIndex]; 
            const j = i * 4;
            imageBuffer[j] = color.r;
            imageBuffer[j + 1] = color.g;
            imageBuffer[j + 2] = color.b;
        }

        ctx.putImageData(imageData, 0, 0)

        window.requestAnimationFrame(update)
    }

    window.requestAnimationFrame(update)
}

const run = async () => {
    // one-time setup
    const canvas = document.getElementById("canvas")
    const menu = document.getElementById("menu")

    const urlParams = new URLSearchParams(window.location.search)
    const gameName = urlParams.get("game")

    if (gameName) {
        const path = `./dist/${gameName}/main.wasm`
        const game = await loadModule(path)
        canvas.classList.remove("hidden")
        await startGame(canvas, game)
    } else {
        const loadUrlButton = document.getElementById("load-url-button")
        const fileInput = document.querySelector("input[type=file]");

        fileInput.addEventListener("change", () => {
            console.log("File Selected")
            const reader = new FileReader()
            const file = fileInput.files[0]
            reader.addEventListener("load", async () => {
                canvas.classList.remove("hidden")
                const game = await loadModule(reader.result);
                await startGame(canvas, game)
                menu.classList.add("hidden")
            })
            reader.readAsDataURL(file)
        })

        loadUrlButton.addEventListener("click", async () => {
            const path = prompt("Enter rom url")
            if (path) {
                const game = await loadModule(path)
                canvas.classList.remove("hidden")
                menu.classList.add("hidden")
                await startGame(canvas, game)
            }
        });

        menu.classList.remove("hidden")
    }
}

run()