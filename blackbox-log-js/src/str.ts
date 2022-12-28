import type { WasmExports } from './wasm';

export type WasmStr = [len: number, ptr: number];

export function getWasmStr([len, ptr]: WasmStr, wasm: WasmExports): string {
	const bytes = new Uint8Array(wasm.memory.buffer, ptr, len);
	const decoder = new TextDecoder('utf-8', {
		// eslint-disable-next-line @typescript-eslint/naming-convention
		ignoreBOM: true,
		fatal: true,
	});
	return decoder.decode(bytes);
}

export function getOptionalWasmStr([len, ptr]: WasmStr, wasm: WasmExports): string | undefined {
	if (ptr === 0) {
		return undefined;
	}

	return getWasmStr([len, ptr], wasm);
}
