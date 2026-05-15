import type { EditorFixture } from "../data/loadFixture";
import type { ComponentFieldMetadata } from "../models/packageMetadata";

export function renderWorkspace(fixture: EditorFixture): string {
  const { packageMetadata, scenarioManifest } = fixture;
  const entity = scenarioManifest.entities[0];

  return `
    <section class="workspace" aria-label="Read-only fixture workspace">
      <header class="toolbar">
        <div>
          <p class="eyebrow">Read-only fixture</p>
          <h2>${scenarioManifest.name}</h2>
        </div>
        <div class="toolbar-actions" aria-label="Unavailable actions">
          <button type="button" disabled>Validate pending</button>
          <button type="button" disabled>Save pending</button>
        </div>
      </header>

      <section class="summary-grid" aria-label="Scenario summary">
        <div>
          <span>Package</span>
          <strong>${packageMetadata.name}</strong>
        </div>
        <div>
          <span>Duration</span>
          <strong>${scenarioManifest.durationSeconds}s</strong>
        </div>
        <div>
          <span>Timestep</span>
          <strong>${scenarioManifest.timestepSeconds}s</strong>
        </div>
      </section>

      <section class="component-panel">
        <div class="panel-head">
          <h2>${entity.label}</h2>
          <span class="quiet">${entity.archetype}</span>
        </div>
        <div class="component-table" role="table" aria-label="Fixture component fields">
          <div class="table-row table-head" role="row">
            <span role="columnheader">Component</span>
            <span role="columnheader">Field</span>
            <span role="columnheader">Value</span>
          </div>
          ${renderComponentRows(fixture)}
        </div>
      </section>

      <section class="component-panel">
        <div class="panel-head">
          <h2>Fixed-step systems</h2>
          <span class="quiet">Read-only graph</span>
        </div>
        <div class="system-list">
          ${packageMetadata.systems
            .map(
              (system) => `
                <article class="system-item">
                  <div>
                    <strong>${system.name}</strong>
                    <span>${system.description}</span>
                  </div>
                  <small>${system.reads.join(", ")} -> ${system.writes.join(", ")}</small>
                </article>
              `,
            )
            .join("")}
        </div>
      </section>

      <section class="algorithm-note" aria-label="Runtime boundary">
        <strong>CLI not connected</strong>
        <span>This page does not validate, save, run, or generate runners.</span>
      </section>
    </section>
  `;
}

function renderComponentRows(fixture: EditorFixture): string {
  const entity = fixture.scenarioManifest.entities[0];

  return fixture.packageMetadata.components
    .flatMap((component) =>
      component.fields.map((field) => renderFieldRow(component.name, field, entity.components[component.name]?.[field.name])),
    )
    .join("");
}

function renderFieldRow(componentName: string, field: ComponentFieldMetadata, value: unknown): string {
  return `
    <div class="table-row" role="row">
      <span>${componentName}</span>
      <span>${field.name} <small>${field.type}${field.unit ? ` / ${field.unit}` : ""}</small></span>
      <span>${String(value ?? field.default)}</span>
    </div>
  `;
}
