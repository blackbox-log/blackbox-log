import { WasmPointer, WasmObject, Module } from './wasm';
import { Log, Headers } from './log';

export class File implements WasmObject {
	readonly #wasm: Module;
	readonly #ptr: WasmPointer;

	#headers: Array<WeakRef<Headers>> = [];
	#logs: Array<WeakRef<Log>> = [];

	constructor(wasm: WebAssembly.Instance & { exports: Module }, data: Uint8Array) {
		this.#wasm = wasm.exports;

		const dataPtr = this.#wasm.data_alloc(data.length);
		if (dataPtr == 0) {
			throw new Error('file is too large');
		}

		const buffer = new Uint8Array(this.#wasm.memory.buffer, dataPtr, data.length);
		buffer.set(data);

		const filePtr = this.#wasm.file_new(dataPtr, data.length);
		this.#ptr = new WasmPointer(filePtr, this.#wasm.file_free);
	}

	free() {
		this.#ptr.free();
	}

	get isAlive(): boolean {
		return this.#ptr.isAlive;
	}

	get logCount(): number {
		const ptr = this.#ptr.ptr;
		return this.#wasm.file_logCount(ptr);
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
