export interface PackageMetadata {
  readonly id: string;
  readonly name: string;
  readonly title: string;
  readonly description: string;
  readonly version: string;
  readonly entryScenarioId: string;
  readonly domain: string;
  readonly tags: readonly string[];
  readonly components: readonly ComponentMetadata[];
  readonly systems: readonly SystemMetadata[];
}

export interface ComponentMetadata {
  readonly name: string;
  readonly description: string;
  readonly fields: readonly ComponentFieldMetadata[];
}

export interface ComponentFieldMetadata {
  readonly name: string;
  readonly type: string;
  readonly unit?: string;
  readonly default: string | number | boolean | readonly number[];
}

export interface SystemMetadata {
  readonly name: string;
  readonly phase: "startup" | "frame" | "fixed" | "cleanup";
  readonly reads: readonly string[];
  readonly writes: readonly string[];
  readonly description: string;
}
