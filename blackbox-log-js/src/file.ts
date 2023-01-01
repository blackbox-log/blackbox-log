import { Headers, Log } from './log';
import { WasmPointer } from './wasm';

import type { WasmExports, WasmObject } from './wasm';

export class File implements WasmObject {
	readonly #wasm: WasmExports;
	readonly #ptr: WasmPointer;

	#headers: Array<WeakRef<Headers>> = [];
	#logs: Array<WeakRef<Log>> = [];

	constructor(wasm: WasmExports, data: Uint8Array) {
		this.#wasm = wasm;

		const dataPtr = this.#wasm.data_alloc(data.length);
		if (dataPtr === 0) {
			throw new Error('file allocation failed');
		}

		const buffer = new Uint8Array(this.#wasm.memory.buffer, dataPtr, data.length);
		buffer.set(data);

		const filePtr = this.#wasm.file_new(dataPtr, data.length);
		this.#ptr = new WasmPointer(filePtr, wasm.file_free);
	}

	free() {
		this.#ptr.free();
	}

	get isAlive(): boolean {
		return this.#ptr.isAlive;
	}

	get logCount(): number {
		return this.#wasm.file_logCount(this.#ptr.ptr);
	}

	parseHeaders(index: number): Headers | undefined {
		if (index >= this.logCount) {
			return undefined;
		}

		if (index in this.#headers && this.#headers[index].deref()?.isAlive) {
			return this.#headers[index].deref();
		}

		const headers = new Headers(this.#wasm, this.#ptr.ptr, index);
		this.#headers[index] = new WeakRef(headers);
		return headers;
	}

	parseLog(index: number): Log | undefined {
		if (index >= this.logCount) {
			return undefined;
		}

		if (this.#logs[index]?.deref()?.isAlive) {
			return this.#logs[index].deref();
		}

		const log = new Log(this.#wasm, this.#ptr.ptr, index);
		this.#logs[index] = new WeakRef(log);
		return log;
	}
}
