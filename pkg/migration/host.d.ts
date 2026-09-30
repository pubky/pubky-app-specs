// The only globals the engine may use: the ones browsers and Node share. The engine compiles
// against these instead of a DOM or Node lib, so a host-only global does not compile. Nothing
// the package declares refers to them, so a consumer needs no host types either.

declare class TextDecoder {
  decode(input?: Uint8Array): string;
}

declare class TextEncoder {
  encode(input?: string): Uint8Array;
}

declare function setTimeout(handler: () => void, ms: number): unknown;
declare function clearTimeout(id: unknown): void;
