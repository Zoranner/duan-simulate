import freeFallPackage from "../fixtures/freeFallPackage.json";
import freeFallScenario from "../fixtures/freeFallScenario.json";
import type { PackageMetadata } from "../models/packageMetadata";
import type { ScenarioManifest } from "../models/scenarioManifest";

export interface EditorFixture {
  readonly packageMetadata: PackageMetadata;
  readonly scenarioManifest: ScenarioManifest;
  readonly boundary: FixtureBoundary;
}

export interface FixtureBoundary {
  readonly mode: "read-only";
  readonly source: string;
  readonly cli: "not connected";
  readonly runner: "not generated";
}

export function loadFixture(): EditorFixture {
  return {
    packageMetadata: freeFallPackage as PackageMetadata,
    scenarioManifest: freeFallScenario as ScenarioManifest,
    boundary: {
      mode: "read-only",
      source: "local JSON fixture",
      cli: "not connected",
      runner: "not generated",
    },
  };
}
