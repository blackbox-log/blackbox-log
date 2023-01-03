import { Memoize as memoize } from 'typescript-memoize';

import { DataParser } from './data';
import { WasmPointer, getOptionalWasmStr, getWasmStr } from './wasm';

import type { WasmExports, WasmObject } from './wasm';

export class Headers implements WasmObject {
	readonly #wasm: WasmExports;
	readonly #ptr: WasmPointer;

	constructor(wasm: WasmExports, file: number, log: number) {
		this.#wasm = wasm;
		const ptr = this.#wasm.file_getHeaders(file, log);
		this.#ptr = new WasmPointer(ptr, wasm.headers_free);
	}

	free() {
		// TODO: free DataParsers too?
		this.#ptr.free();
	}

	get isAlive(): boolean {
		return this.#ptr.isAlive;
	}

	getDataParser(): DataParser {
		const ptr = this.#wasm.headers_getDataParser(this.#ptr.ptr);
		return new DataParser(this.#wasm, this, ptr);
	}

	@memoize()
	get firmwareRevision(): string {
		const revision = this.#wasm.headers_firmwareRevision(this.#ptr.ptr);
		return getWasmStr(revision, this.#wasm);
	}

	@memoize()
	get boardInfo(): string | undefined {
		const name = this.#wasm.headers_boardInfo(this.#ptr.ptr);
		return getOptionalWasmStr(name, this.#wasm);
	}

	@memoize()
	get craftName(): string | undefined {
		const name = this.#wasm.headers_craftName(this.#ptr.ptr);
		return getOptionalWasmStr(name, this.#wasm);
	}
}
