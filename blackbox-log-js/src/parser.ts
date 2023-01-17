import { File } from './file';
import { getWasmStr } from './str';
import { type WasmExports } from './wasm';

export type WasmInit = string | URL | Request | Response | WebAssembly.Module;

type Instance = WebAssembly.Instance & { exports: WasmExports };

export default class Parser {
	static async init(init: WasmInit): Promise<Parser> {
		let instance: Instance | undefined;

		const imports = {
			main: {
				panic(len: number, ptr: number) {
					if (instance === undefined) {
						console.error('received panic before JS handler was initialized');
						return;
					}

					console.error(getWasmStr([len, ptr], instance.exports));
				},
				throw(len: number, ptr: number) {
					if (instance === undefined) {
						throw new ParseError('unknown error');
					} else {
						const message = getWasmStr([len, ptr], instance.exports);
						instance.exports.slice8_free(len, ptr);
						throw new ParseError(message);
					}
				},
			},
		};

		if (init instanceof WebAssembly.Module) {
			instance = new WebAssembly.Instance(init, imports) as Instance;
		} else {
			const response = init instanceof Response ? init : fetch(init);

			const streamed = await WebAssembly.instantiateStreaming(response, imports);
			instance = streamed.instance as Instance;
		}

		(instance.exports.set_panic_hook as () => void)();

		return new Parser(instance as { exports: WasmExports });
	}

	readonly #wasm: WasmExports;

	constructor(wasm: WebAssembly.Instance) {
		this.#wasm = wasm.exports as WasmExports;
	}

	loadFile(data: Uint8Array): File {
		return new File(this.#wasm, data);
	}
}

export class ParseError extends Error {
	constructor(message: string, options?: ErrorOptions) {
		super(message, options);

		// Maintain V8 stack trace
		// @ts-expect-error Only present on V8 and is missing from typedef
		Error.captureStackTrace?.(this, ParseError); // eslint-disable-line @typescript-eslint/no-unsafe-call

		this.name = 'ParseError';
	}
}
