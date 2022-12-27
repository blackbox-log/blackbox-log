export interface WasmObject {
	free(): void;
	isAlive: boolean;
}

export type Module = {
	memory: WebAssembly.Memory;

	data_alloc: (length: number) => number;
	str_free: (ptr: number) => void;

	file_free: (ptr: number) => void;
	file_new: (ptr: number, length: number) => number;
	file_logCount: (ptr: number) => number;
	file_getHeaders: (ptr: number, log: number) => number;
	file_getLog: (ptr: number, log: number) => number;

	headers_free: (ptr: number) => void;
	headers_firmwareRevision: (ptr: number, isLog: boolean) => number;
	headers_boardInfo: (ptr: number, isLog: boolean) => number;
	headers_craftName: (ptr: number, isLog: boolean) => number;

	log_free: (ptr: number) => void;
	log_mainFrameCount: (ptr: number) => number;
	log_gpsFrameCount: (ptr: number) => number;
};

export class WasmPointer {
	#ptr?: number;
	readonly #free;

	static #dealloc = ({ ptr, free }: { ptr: number; free: (ptr: number) => void }) => {
		console.log('running dealloc...');
		free(ptr);
	};

	#registry = new FinalizationRegistry(WasmPointer.#dealloc);

	constructor(ptr: number, free: (ptr: number) => void) {
		this.#ptr = ptr;
		this.#free = free;
		this.#registry.register(this, { ptr, free }, this);
	}

	free() {
		if (this.#ptr != null) {
			this.#free(this.#ptr);
			this.#registry.unregister(this);
			this.#ptr = undefined;
		}
	}

	get isAlive(): boolean {
		return this.#ptr != null;
	}

	get ptr(): number {
		if (this.#ptr == null) {
			throw new Error('backing WebAssembly object has been freed');
		}

		return this.#ptr;
	}
}
