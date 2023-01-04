import { WasmPointer } from './wasm';

import type { Headers } from './headers';
import type { WasmExports, WasmObject } from './wasm';

export class DataParser implements WasmObject {
	readonly #wasm: WasmExports;
	readonly #ptr: WasmPointer;
	readonly #headers: Headers;

	constructor(wasm: WasmExports, headers: Headers, ptr: number) {
		this.#wasm = wasm;
		this.#headers = headers;
		this.#ptr = new WasmPointer(ptr, wasm.data_free);
	}

	free() {
		this.#ptr.free();
	}

	get isAlive(): boolean {
		return this.#ptr.isAlive;
	}

	get headers(): Headers {
		return this.#headers;
	}

	get mainFrameCount(): number {
		return this.#wasm.data_mainFrameCount(this.#ptr.ptr);
	}

	get gpsFrameCount(): number {
		return this.#wasm.data_gpsFrameCount(this.#ptr.ptr);
	}
}
