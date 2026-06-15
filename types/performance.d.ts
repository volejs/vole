/**
 * Performance API declarations for Vole global `performance`.
 *
 * Vole aligns with the Web Performance interface for timing-related helpers.
 */
declare global {
  /**
   * The global `performance` object.
   */
  var performance: Performance;

  /**
   * Standard timing entry shared by `PerformanceMark` / `PerformanceMeasure`.
   */
  interface PerformanceEntry {
    /** Entry name. */
    name: string;
    /** Entry type, usually `mark` or `measure`. */
    entryType: string;
    /** Entry start timestamp, in milliseconds since `timeOrigin`. */
    startTime: number;
    /** Entry duration, in milliseconds. */
    duration: number;
    /** Marker/detail extension point for user-defined payload. */
    detail?: unknown;
    /** Serialize the entry as a plain object. */
    toJSON(): {
      name: string;
      entryType: string;
      startTime: number;
      duration: number;
      detail?: unknown;
    };
  }

  /**
   * Mark entry.
   */
  interface PerformanceMark extends PerformanceEntry {
    /**
     * Alias to `"mark"`.
     */
    readonly entryType: "mark";
  }

  /**
   * Measure entry.
   */
  interface PerformanceMeasure extends PerformanceEntry {
    /**
     * Alias to `"measure"`.
     */
    readonly entryType: "measure";
  }

  interface Performance {
    /**
     * High-resolution monotonic timestamp since `timeOrigin`, in milliseconds.
     */
    now(): number;
    /**
     * Creates a performance mark.
     */
    mark(name: string, options?: PerformanceMarkOptions): PerformanceMark;
    /**
     * Creates a performance measure.
     */
    measure(name: string, start?: string | number, end?: string | number): PerformanceMeasure;
    /**
     * Creates a performance measure from option object.
     */
    measure(name: string, options: PerformanceMeasureOptions): PerformanceMeasure;

    clearMarks(name?: string): void;
    clearMeasures(name?: string): void;
    getEntries(): PerformanceEntry[];
    getEntriesByType(type: string): PerformanceEntry[];
    getEntriesByName(name: string, type?: string): PerformanceEntry[];

    /**
     * Web standard `timeOrigin` value (Unix epoch in ms at context bootstrap).
     */
    readonly timeOrigin: number;
    /**
     * Additional metadata form; Vole uses `{ timeOrigin }`.
     */
    toJSON(): { timeOrigin: number };
  }

  interface PerformanceMarkOptions {
    /**
     * Optional mark timestamp override.
     */
    startTime?: number;
    /**
     * Optional mark detail payload.
     */
    detail?: unknown;
  }

  interface PerformanceMeasureOptions {
    /**
     * Start boundary, as timestamp or mark name.
     */
    start?: string | number;
    /**
     * End boundary, as timestamp or mark name.
     */
    end?: string | number;
    /**
     * Optional measure detail payload.
     */
    detail?: unknown;
  }
}

export {};
