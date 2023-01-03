import Parser from './parser';

import type { File } from './file';
import type { DataParser, HeaderView, Headers } from './log';
import type { WasmObject } from './wasm';

export default Parser;
export type { File, HeaderView, Headers, DataParser as Log, WasmObject };
