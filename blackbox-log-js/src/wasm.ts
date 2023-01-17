import type { WasmStr } from './str';

export type WasmObject = {
	isAlive: boolean;
	free(): void;
};

export type WasmExports = {
	memory: WebAssembly.Memory;

	data_alloc: (length: number) => number;
	slice8_free: (length: number, ptr: number) => void;

	file_free: (ptr: number) => void;
	file_new: (ptr: number, length: number) => number;
	file_logCount: (ptr: number) => number;
	file_getHeaders: (ptr: number, log: number) => number;
	file_getLog: (ptr: number, log: number) => number;

	headers_free: (ptr: number) => void;
	headers_getDataParser: (ptr: number) => number;
	headers_mainDef: (ptr: number) => number;
	headers_slowDef: (ptr: number) => number;
	headers_gpsDef: (ptr: number) => number;
	headers_firmwareRevision: (ptr: number) => WasmStr;
	headers_boardInfo: (ptr: number) => WasmStr;
	headers_craftName: (ptr: number) => WasmStr;

	frameDef_free: (ptr: number) => void;

	data_free: (ptr: number) => void;
	data_resultPtr: (ptr: number) => number;
	data_counts: (ptr: number) => [number, number, number, number, number];
	data_next: (ptr: number) => void;
};

const registry = new FinalizationRegistry(dealloc);

function dealloc({ ptr, free }: { ptr: number; free: (ptr: number) => void }) {
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
