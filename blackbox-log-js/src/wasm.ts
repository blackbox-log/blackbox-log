import { getOptionalWasmStr, getWasmStr } from './str';

import type { WasmStr } from './str';

export { getWasmStr, getOptionalWasmStr };

export type WasmObject = {
	isAlive: boolean;
	free(): void;
};

export type WasmExports = {
	memory: WebAssembly.Memory;

	data_alloc: (length: number) => number;

	file_free: (ptr: number) => void;
	file_new: (ptr: number, length: number) => number;
	file_logCount: (ptr: number) => number;
	file_getHeaders: (ptr: number, log: number) => number;
	file_getLog: (ptr: number, log: number) => number;

	headers_free: (ptr: number) => void;
	headers_firmwareRevision: (ptr: number, isLog: boolean) => WasmStr;
	headers_boardInfo: (ptr: number, isLog: boolean) => WasmStr;
	headers_craftName: (ptr: number, isLog: boolean) => WasmStr;

	log_free: (ptr: number) => void;
	log_mainFrameCount: (ptr: number) => number;
	log_gpsFrameCount: (ptr: number) => number;
};

const registry = new FinalizationRegistry(dealloc);

function dealloc({ ptr, free }: { ptr: number; free: (ptr: number) => void }) {
	console.log(`running dealloc for ${ptr}`);
	free(ptr);
}

export class WasmPointer {
	#ptr: number | undefined;
	readonly #free;

	constructor(ptr: number, free: (ptr: number) => void) {
		this.#ptr = ptr;
		this.#free = free;
		registry.register(this, { ptr, free }, this);
	}

	free() {
		if (this.#ptr !== undefined) {
			this.#free(this.#ptr);
			registry.unregister(this);
			this.#ptr = undefined;
		}
	}

	get isAlive(): boolean {
		return this.#ptr !== undefined;
	}

	get ptr(): number {
		if (this.#ptr === undefined) {
			throw new Error('backing WebAssembly object has been freed');
		}

		return this.#ptr;
	}
}
