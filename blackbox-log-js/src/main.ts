import encodedWasm from './blackbox-log.wasm?inline';
import Parser from './parser';

import type { WasmExports } from './wasm';

export * from './common';

export default class SimpleParser extends Parser {
	static async init(): Promise<SimpleParser> {
		const decoded = atob(encodedWasm);

		const bytes = new Uint8Array(decoded.length);
		for (let i = 0; i < decoded.length; i++) {
			bytes[i] = decoded.charCodeAt(i);
		}

		const { instance } = await WebAssembly.instantiate(bytes);
		return new SimpleParser(instance as { exports: WasmExports });
	}

	private constructor(wasm: WebAssembly.Instance & { exports: WasmExports }) {
		if (!(wasm instanceof WebAssembly.Instance)) {
			throw new Error('create a SimpleParser using its init() method, not new');
		}

		super(wasm);
	}
}
