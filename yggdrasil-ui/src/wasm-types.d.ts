declare module './wasmLoader' {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  export default function init(module?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module): Promise<unknown>;
  export class WasmEngine {
    constructor();
    loadClientFeatures(featuresJson: string): unknown;
    listToggles(): unknown;
    getGrammars(): unknown;
    getToggleGrammar(toggleName: string): string | undefined;
    setToggleGrammar(toggleName: string, grammar: string): void;
    setGrammars(value: Record<string, string>): void;
    evaluate(toggleName: string, contextJson: string): unknown;
  }
}
