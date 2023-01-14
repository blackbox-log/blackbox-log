import { Memoize as memoize } from 'typescript-memoize';

import { DataParser } from './data';
import { getOptionalWasmStr, getWasmStr } from './str';
import { WasmPointer } from './wasm';

import type { WasmExports, WasmObject } from './wasm';

export type FrameDef = FieldDef[];
export type FieldDef = {
	name: string;
	signed: boolean;
};

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
	get mainFrameDef(): FrameDef {
		const ptr = this.#wasm.headers_mainDef(this.#ptr.ptr);
		return getFrameDef(ptr, this.#wasm);
	}

	@memoize()
	get slowFrameDef(): FrameDef {
		const ptr = this.#wasm.headers_slowDef(this.#ptr.ptr);
		return getFrameDef(ptr, this.#wasm);
	}

	@memoize()
	get gpsFrameDef(): FrameDef {
		const ptr = this.#wasm.headers_gpsDef(this.#ptr.ptr);
		return getFrameDef(ptr, this.#wasm);
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

function getFrameDef(ptr: number, wasm: WasmExports): FrameDef {
	const fieldDefLength = 3;

	const [len, fields] = new Uint32Array(wasm.memory.buffer, ptr, 2);

	const def = [];
	if (len !== 0 && fields !== 0) {
		const data = new Uint32Array(wasm.memory.buffer, fields, len * fieldDefLength);

		for (let field = 0; field < len; field++) {
			const start = field * fieldDefLength;

			def.push({
				name: getWasmStr([data[start], data[start + 1]], wasm),
				signed: data[start + 2] !== 0,
			});
		}
	}

	wasm.frameDef_free(ptr);

	return def;
}
