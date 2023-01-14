import { File } from './file';
import { getWasmStr } from './str';

import type { WasmExports } from './wasm';

export type WasmInit = string | URL | Request | Response | WebAssembly.Module;

export default class Parser {
	static async init(init: WasmInit): Promise<Parser> {
		let instance: WebAssembly.Instance | undefined;

		const imports = {
			main: {
				panic(len: number, ptr: number) {
					if (instance === undefined) {
						console.error('received panic before JS handler was initialized');
						return;
					}

					console.error(getWasmStr([len, ptr], instance.exports as WasmExports));
				},
			},
		};

		if (init instanceof WebAssembly.Module) {
			instance = new WebAssembly.Instance(init, imports);
		} else {
			const response = init instanceof Response ? init : fetch(init);

			const streamed = await WebAssembly.instantiateStreaming(response, imports);
			instance = streamed.instance;
		}

		(instance.exports.set_panic_hook as () => void)();

		return new Parser(instance as { exports: WasmExports });
	}

	readonly #wasm: WasmExports;

	constructor(wasm: WebAssembly.Instance & { exports: WasmExports }) {
		this.#wasm = wasm.exports;
	}

	loadFile(data: Uint8Array): File {
		return new File(this.#wasm, data);
	}
}
