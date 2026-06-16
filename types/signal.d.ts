/**
 * Process signaling helpers exposed by `vole:signal`.
 *
 * Provides both signal sending and receiving semantics:
 * - send/kill for process notification
 * - signal stream for observing OS signals in async iteration mode
 * - EventTarget integration via `SignalStream` events (`signal`/`error`/`close`).
 */
declare module "vole:signal" {
  /** Send a signal to a process id and return whether dispatch was accepted. */
  export function sendSignal(pid: number, signal?: string): boolean;

  /** Alias of `sendSignal`. */
  export function killProcess(pid: number, signal?: string): boolean;

  /** Return supported signal names by this runtime. */
  export function listSignals(): string[];

  /** Check whether a signal name is recognized by this runtime. */
  export function isSupported(signalName: string): boolean;

  /**
   * Subscribe to OS signals and consume them as EventTarget / AsyncIterator.
   */
  export class SignalStream extends EventTarget {
    /**
     * Create a new subscription stream.
     *
     * @param signals Optional list of signal names. When omitted, subscribe to all
     *   supported signals.
     */
    constructor(signals?: readonly string[]);

    /**
     * Close subscription explicitly.
     */
    close(): void;

    /**
     * Read one signal from the async stream.
     */
    next(): Promise<IteratorResult<string>>;

    /**
     * Async iterator entry, so `for await (const signal of stream())` is supported.
     */
    [Symbol.asyncIterator](): AsyncIterator<string>;
  }

  /**
   * Create and return a `SignalStream` instance directly.
   */
  export function stream(signals?: readonly string[]): SignalStream;

  /** Default object-style export. */
  const signal: {
    sendSignal: typeof sendSignal;
    killProcess: typeof killProcess;
    listSignals: typeof listSignals;
    isSupported: typeof isSupported;
    stream: typeof stream;
    SignalStream: typeof SignalStream;
  };

  export default signal;
}
