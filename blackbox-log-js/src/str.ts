import { Module } from './wasm';

export function getWasmStr(ptr: number, wasm: Module): string {
	try {
		const str = new Uint32Array(wasm.memory.buffer, ptr, 2);
		const bytes = new Uint8Array(wasm.memory.buffer, str[1], str[0]);
		const decoder = new TextDecoder('utf-8', {
			ignoreBOM: true,
			fatal: true,
		});
		return decoder.decode(bytes);
	} finally {
		wasm.str_free(ptr);
	}
}

export function getOptionalWasmStr(ptr: number, wasm: Module): string | undefined {
	if (ptr == 0) {
		return undefined;
	}

	return getWasmStr(ptr, wasm);
}
