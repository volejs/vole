/**
 * Runtime lifecycle extension helpers exposed by `vole:until`.
 *
 * `WaitUntil` tracks background work that should complete before a scope or the
 * runtime finishes. Managed scopes join the runtime registry; detached scopes are
 * only visible to the caller that owns them.
 *
 * @example
 * ```ts
 * import { WaitUntil, drainManaged } from "vole:until";
 *
 * const until = new WaitUntil();
 * until.waitUntil(Promise.resolve().then(() => console.log("background")));
 *
 * await drainManaged(30_000);
 * ```
 *
 * @example Detached scope
 * ```ts
 * const local = new WaitUntil({ managed: false });
 * local.waitUntil(Promise.resolve());
 * await local.drain(1_000);
 * ```
 *
 * @remarks
 * `waitUntil` is not a cancellation primitive. Rejections are reported through
 * `reportError` and still release the pending guard. Closing a scope prevents
 * new work from being registered but does not cancel work already running.
 */
declare module "vole:until" {
  /** Options used when creating a `WaitUntil` scope. */
  export interface WaitUntilOptions {
    /** Whether the scope joins the runtime-level registry. Defaults to `true`. */
    managed?: boolean;
  }

  /** Lifecycle scope that tracks background promises through guard accounting. */
  export class WaitUntil {
    /**
     * Create a new lifecycle scope.
     *
     * @param options Scope creation options. Omit for a managed scope.
     */
    constructor(options?: WaitUntilOptions);
    /** Number of currently pending promises tracked by this scope. */
    readonly pendingCount: number;
    /** Whether this scope currently has no pending work. */
    readonly isIdle: boolean;
    /** Whether this scope no longer accepts new work. */
    readonly isClosed: boolean;
    /** Whether this scope is registered with the runtime-level registry. */
    readonly managed: boolean;
    /**
     * Track a promise-like value and report rejection through `reportError`.
     *
     * @param promise Work item to track. Non-promise values are wrapped with `Promise.resolve`.
     */
    waitUntil(promise: PromiseLike<unknown> | unknown): void;
    /**
     * Wait for this scope to become idle or reject on timeout.
     *
     * @param timeout Timeout in milliseconds. Defaults to the runtime's wait-until timeout.
     */
    drain(timeout?: number): Promise<void>;
    /** Close this scope to prevent new work from being registered. */
    close(): void;
  }

  /** Track work in a specific scope. */
  export function waitUntil(target: WaitUntil, promise: PromiseLike<unknown> | unknown): void;
  /** Drain one scope until it is idle. */
  export function drain(target: WaitUntil, timeout?: number): Promise<void>;
  /** Close one scope. */
  export function close(target: WaitUntil): void;
  /** Return pending work count for one scope. */
  export function pendingCount(target: WaitUntil): number;
  /** Return whether one scope is idle. */
  export function isIdle(target: WaitUntil): boolean;
  /** Return whether one scope is closed. */
  export function isClosed(target: WaitUntil): boolean;
  /** Return whether one scope is managed by the runtime registry. */
  export function isManaged(target: WaitUntil): boolean;
  /** Drain all managed scopes registered in the current runtime context. */
  export function drainManaged(timeout?: number): Promise<void>;
  /** Close the runtime-level managed registry. */
  export function closeManaged(): void;
  /** Return pending work count across all managed scopes. */
  export function managedPendingCount(): number;
  /** Return live managed scope count. */
  export function managedScopeCount(): number;
  /** Return whether all managed scopes are idle. */
  export function managedIsIdle(): boolean;

  /** Default object export containing all wait-until helpers. */
  const until: {
    /** WaitUntil class constructor. */
    WaitUntil: typeof WaitUntil;
    /** See `waitUntil`. */
    waitUntil: typeof waitUntil;
    /** See `drain`. */
    drain: typeof drain;
    /** See `close`. */
    close: typeof close;
    /** See `pendingCount`. */
    pendingCount: typeof pendingCount;
    /** See `isIdle`. */
    isIdle: typeof isIdle;
    /** See `isClosed`. */
    isClosed: typeof isClosed;
    /** See `isManaged`. */
    isManaged: typeof isManaged;
    /** See `drainManaged`. */
    drainManaged: typeof drainManaged;
    /** See `closeManaged`. */
    closeManaged: typeof closeManaged;
    /** See `managedPendingCount`. */
    managedPendingCount: typeof managedPendingCount;
    /** See `managedScopeCount`. */
    managedScopeCount: typeof managedScopeCount;
    /** See `managedIsIdle`. */
    managedIsIdle: typeof managedIsIdle;
  };
  export default until;
}
