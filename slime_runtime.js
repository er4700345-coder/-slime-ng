// runtime/slime_runtime.js
// SLIME Host Runtime — provides the "slime" import namespace to WASM modules.
// Drop this in any JS environment (browser or Node) to run .wasm output.

"use strict";

const SlimeRuntime = (() => {
    // Shared memory reference — set when module is instantiated
    let _memory = null;

    // Decode a UTF-8 string from WASM linear memory
    function readString(ptr, len) {
        if (!_memory) throw new Error("SlimeRuntime: memory not initialized");
        const bytes = new Uint8Array(_memory.buffer, ptr, len);
        return new TextDecoder("utf-8").decode(bytes);
    }

    // The import object passed to WebAssembly.instantiate
    const imports = {
        slime: {
            // ── I/O ──────────────────────────────────────────────────────
            print(ptr, len) {
                process.stdout
                    ? process.stdout.write(readString(ptr, len))
                    : console.log(readString(ptr, len));
            },

            println(ptr, len) {
                const s = readString(ptr, len);
                process.stdout
                    ? process.stdout.write(s + "\n")
                    : console.log(s);
            },

            print_i32(value) {
                process.stdout
                    ? process.stdout.write(String(value))
                    : console.log(value);
            },

            print_f64(value) {
                process.stdout
                    ? process.stdout.write(String(value))
                    : console.log(value);
            },

            // ── Panic ────────────────────────────────────────────────────
            panic(ptr, len) {
                const msg = readString(ptr, len);
                console.error(`\n[SLIME PANIC]: ${msg}`);
                if (typeof process !== "undefined") process.exit(1);
                else throw new Error(`SLIME panic: ${msg}`);
            },
        },
    };

    // Instantiate a .wasm binary (ArrayBuffer or Uint8Array)
    async function instantiate(wasmBytes) {
        const result = await WebAssembly.instantiate(wasmBytes, imports);
        const instance = result.instance;

        // Wire up memory export so host functions can read strings
        if (instance.exports.memory) {
            _memory = instance.exports.memory;
        }

        return instance;
    }

    // Node.js runner — load a .wasm file and call main()
    async function runFile(wasmPath) {
        const fs = await import("fs");
        const bytes = fs.readFileSync(wasmPath);
        const instance = await instantiate(bytes);

        if (!instance.exports.main) {
            console.error("SlimeRuntime: no 'main' export found in module");
            process.exit(1);
        }

        const exitCode = instance.exports.main();
        if (typeof exitCode === "number" && exitCode !== 0) {
            process.exit(exitCode);
        }
    }

    return { imports, instantiate, runFile };
})();

// Node CLI: `node slime_runtime.js program.wasm`
if (typeof process !== "undefined" && process.argv[2]) {
    SlimeRuntime.runFile(process.argv[2]).catch(err => {
        console.error("Runtime error:", err.message);
        process.exit(1);
    });
}

if (typeof module !== "undefined") module.exports = SlimeRuntime;
