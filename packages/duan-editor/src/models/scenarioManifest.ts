export interface ScenarioManifest {
  readonly id: string;
  readonly name: string;
  readonly packageId: string;
  readonly description: string;
  readonly durationSeconds: number;
  readonly timestepSeconds: number;
  readonly entities: readonly ScenarioEntity[];
  readonly events: readonly ScenarioEvent[];
  readonly expectations: readonly ScenarioExpectation[];
}

export interface ScenarioEntity {
  readonly id: string;
  readonly archetype: string;
  readonly label: string;
  readonly components: Record<string, Record<string, unknown>>;
}

export interface ScenarioEvent {
  readonly atSeconds: number;
  readonly label: string;
  readonly description: string;
}

export interface ScenarioExpectation {
  readonly label: string;
  readonly value: string;
}
