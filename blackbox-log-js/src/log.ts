import { Memoize as memoize } from 'typescript-memoize';

import { WasmPointer, getOptionalWasmStr, getWasmStr } from './wasm';

import type { WasmExports, WasmObject } from './wasm';

export type HeaderView = {
	firmwareRevision: string;
	boardInfo: string | undefined;
	craftName: string | undefined;
};

export class Headers implements HeaderView, WasmObject {
	readonly #wasm: WasmExports;
	readonly #ptr: WasmPointer;

	constructor(wasm: WasmExports, file: number, log: number) {
		this.#wasm = wasm;
		const ptr = this.#wasm.file_getHeaders(file, log);
		this.#ptr = new WasmPointer(ptr, wasm.headers_free);
	}

	free() {
		this.#ptr.free();
	}

	get isAlive(): boolean {
		return this.#ptr.isAlive;
	}

	@memoize()
	get firmwareRevision(): string {
		const revision = this.#wasm.headers_firmwareRevision(this.#ptr.ptr, false);
		return getWasmStr(revision, this.#wasm);
	}

	@memoize()
	get boardInfo(): string | undefined {
		const name = this.#wasm.headers_boardInfo(this.#ptr.ptr, false);
		return getOptionalWasmStr(name, this.#wasm);
	}

	@memoize()
	get craftName(): string | undefined {
		const name = this.#wasm.headers_craftName(this.#ptr.ptr, false);
		return getOptionalWasmStr(name, this.#wasm);
	}
}

export class Log implements HeaderView, WasmObject {
	readonly #wasm: WasmExports;
	readonly #ptr: WasmPointer;

	constructor(wasm: WasmExports, file: number, log: number) {
		this.#wasm = wasm;
		const ptr = this.#wasm.file_getLog(file, log);
		this.#ptr = new WasmPointer(ptr, wasm.log_free);
	}

	free() {
		this.#ptr.free();
	}

	get isAlive(): boolean {
		return this.#ptr.isAlive;
	}

	@memoize()
	get firmwareRevision(): string {
		const revision = this.#wasm.headers_firmwareRevision(this.#ptr.ptr, true);
		return getWasmStr(revision, this.#wasm);
	}

	@memoize()
	get boardInfo(): string | undefined {
		const name = this.#wasm.headers_boardInfo(this.#ptr.ptr, true);
		return getOptionalWasmStr(name, this.#wasm);
	}

	@memoize()
	get craftName(): string | undefined {
		const name = this.#wasm.headers_craftName(this.#ptr.ptr, true);
		return getOptionalWasmStr(name, this.#wasm);
	}

	@memoize()
	get mainFrameCount(): number {
		return this.#wasm.log_mainFrameCount(this.#ptr.ptr);
	}

	@memoize()
	get gpsFrameCount(): number {
		return this.#wasm.log_gpsFrameCount(this.#ptr.ptr);
	}
}
