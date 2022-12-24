import init from './blackbox-log.wasm?init';
import { Memoize } from 'typescript-memoize';

export { init };

type Module = {
  memory: WebAssembly.Memory;

  data_alloc: (length: number) => number;
  str_free: (ptr: number) => void;

  file_free: (ptr: number) => void;
  file_new: (ptr: number, length: number) => number;
  file_logCount: (ptr: number) => number;
  file_getHeaders: (ptr: number, log: number) => number;
  file_getLog: (ptr: number, log: number) => number;

  headers_free: (ptr: number) => void;
  headers_firmwareRevision: (ptr: number, isLog: boolean) => number;
  headers_boardInfo: (ptr: number, isLog: boolean) => number;
  headers_craftName: (ptr: number, isLog: boolean) => number;

  log_free: (ptr: number) => void;
  log_mainFrameCount: (ptr: number) => number;
  log_gpsFrameCount: (ptr: number) => number;
}

function getWasmStr(ptr: number, wasm: Module): string {
  try {
    const str = new Uint32Array(wasm.memory.buffer, ptr, 2);
    const bytes = new Uint8Array(wasm.memory.buffer, str[1], str[0]);
    const decoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
    return decoder.decode(bytes);
  } finally {
    wasm.str_free(ptr);
  }
}

function getOptionalWasmStr(ptr: number, wasm: Module): string | undefined {
  if (ptr == 0) {
    return undefined;
  }

  return getWasmStr(ptr, wasm);
}

class WasmPointer {
  #ptr?: number;
  readonly #free;

  static #dealloc = ({ ptr, free }: { ptr: number, free: (ptr: number) => void }) => {
    console.log('running dealloc...');
    free(ptr)
  };

  #registry = new FinalizationRegistry(WasmPointer.#dealloc);

  constructor(ptr: number, free: (ptr: number) => void) {
    this.#ptr = ptr;
    this.#free = free;
    this.#registry.register(this, { ptr, free }, this);
  }

  free() {
    if (this.#ptr != null) {
      this.#free(this.#ptr);
      this.#registry.unregister(this);
      this.#ptr = undefined;
    }
  }

  get isAlive(): boolean {
    return this.#ptr != null;
  }

  get ptr(): number {
    if (this.#ptr == null) {
      throw new Error('backing WebAssembly object has been freed');
    }

    return this.#ptr;
  }
}

interface WasmObject {
  free(): void;
  isAlive: boolean;
}

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

interface HeaderView {
  firmwareRevision: string;
  boardInfo?: string;
  craftName?: string;
}

export class Headers implements HeaderView, WasmObject {
  readonly #wasm: Module;
  readonly #ptr: WasmPointer;

  constructor(wasm: Module, file: number, log: number) {
    this.#wasm = wasm;
    const ptr = this.#wasm.file_getHeaders(file, log);
    this.#ptr = new WasmPointer(ptr, this.#wasm.headers_free);
  }

  free() {
    this.#ptr.free();
  }

  get isAlive(): boolean {
    return this.#ptr.isAlive;
  }

  @Memoize()
  get firmwareRevision(): string {
    const revision = this.#wasm.headers_firmwareRevision(this.#ptr.ptr, false);
    return getWasmStr(revision, this.#wasm);
  }

  @Memoize()
  get boardInfo(): string | undefined {
    const name = this.#wasm.headers_boardInfo(this.#ptr.ptr, false);
    return getOptionalWasmStr(name, this.#wasm);
  }

  @Memoize()
  get craftName(): string | undefined {
    const name = this.#wasm.headers_craftName(this.#ptr.ptr, false);
    return getOptionalWasmStr(name, this.#wasm);
  }
}

export class Log implements HeaderView, WasmObject {
  readonly #wasm: Module;
  readonly #ptr: WasmPointer;

  constructor(wasm: Module, file: number, log: number) {
    this.#wasm = wasm;
    const ptr = this.#wasm.file_getLog(file, log);
    this.#ptr = new WasmPointer(ptr, this.#wasm.log_free);
  }

  free() {
    this.#ptr.free();
  }

  get isAlive(): boolean {
    return this.#ptr.isAlive;
  }

  @Memoize()
  get firmwareRevision(): string {
    const revision = this.#wasm.headers_firmwareRevision(this.#ptr.ptr, true);
    return getWasmStr(revision, this.#wasm);
  }

  @Memoize()
  get boardInfo(): string | undefined {
    const name = this.#wasm.headers_boardInfo(this.#ptr.ptr, true);
    return getOptionalWasmStr(name, this.#wasm);
  }

  @Memoize()
  get craftName(): string | undefined {
    const name = this.#wasm.headers_craftName(this.#ptr.ptr, true);
    return getOptionalWasmStr(name, this.#wasm);
  }

  @Memoize()
  get mainFrameCount(): number {
    return this.#wasm.log_mainFrameCount(this.#ptr.ptr);
  }

  @Memoize()
  get gpsFrameCount(): number {
    return this.#wasm.log_gpsFrameCount(this.#ptr.ptr);
  }
}
