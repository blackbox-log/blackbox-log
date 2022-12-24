import { Memoize } from 'typescript-memoize';
import { WasmPointer, WasmObject, Module } from './wasm';
import { getOptionalWasmStr, getWasmStr } from './str';

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
