import { File } from './file';

import type { WasmExports } from './wasm';

export type WasmInit = string | URL | Request | Response | WebAssembly.Module;

export default class Parser {
	static async init(wasm: WasmInit): Promise<Parser> {
		if (wasm instanceof WebAssembly.Module) {
			wasm = new WebAssembly.Instance(wasm);
		} else {
			if (!(wasm instanceof Response)) {
				wasm = fetch(wasm);
			}

			const streamed = await WebAssembly.instantiateStreaming(
				wasm as Promise<Response> | Response,
			);
			wasm = streamed.instance;
		}

		return new Parser(wasm as WebAssembly.Instance);
	}

	readonly #wasm: WasmExports;

	constructor(wasm: WebAssembly.Instance) {
		this.#wasm = wasm.exports as WasmExports;
	}

	loadFile(data: Uint8Array): File {
		return new File(this.#wasm, data);
	}
}
