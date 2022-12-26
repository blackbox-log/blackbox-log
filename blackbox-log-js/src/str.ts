import type { WasmExports } from './wasm';

export function getWasmStr(ptr: number, wasm: WasmExports): string {
	try {
		const [len, strPtr] = new Uint32Array(wasm.memory.buffer, ptr, 2);
		const bytes = new Uint8Array(wasm.memory.buffer, strPtr, len);
		const decoder = new TextDecoder('utf-8', {
			// eslint-disable-next-line @typescript-eslint/naming-convention
			ignoreBOM: true,
			fatal: true,
		});
		return decoder.decode(bytes);
	} finally {
		wasm.str_free(ptr);
	}
}

export function getOptionalWasmStr(ptr: number, wasm: WasmExports): string | undefined {
	if (ptr === 0) {
		return undefined;
	}

	return getWasmStr(ptr, wasm);
}
