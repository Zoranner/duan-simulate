import type { EditorFixture } from "../data/loadFixture";

export function renderStatusPanel(fixture: EditorFixture): string {
  const { packageMetadata, scenarioManifest, boundary } = fixture;

  return `
    <aside class="run-panel" aria-label="Read-only status">
      <header class="toolbar compact">
        <div>
          <p class="eyebrow">Status</p>
          <h2>Runner not generated</h2>
        </div>
        <button type="button" disabled>Run pending</button>
      </header>

      <section class="metric-grid">
        <div>
          <span>Mode</span>
          <strong>Read-only</strong>
        </div>
        <div>
          <span>CLI</span>
          <strong>Not connected</strong>
        </div>
      </section>

      <section class="output-panel">
        <div class="panel-head">
          <h2>Fixture output</h2>
          <span class="status-dot">No run</span>
        </div>
        <pre>${JSON.stringify(
          {
            package: packageMetadata.id,
            scenario: scenarioManifest.id,
            source: boundary.source,
            cli: boundary.cli,
            runner: boundary.runner,
          },
          null,
          2,
        )}</pre>
      </section>

      <section class="output-panel">
        <div class="panel-head">
          <h2>Events</h2>
          <span class="quiet">Fixture notes</span>
        </div>
        <ol class="event-list">
          ${scenarioManifest.events
            .map((event) => `<li><strong>${event.atSeconds}s</strong> ${event.label}: ${event.description}</li>`)
            .join("")}
        </ol>
      </section>

      <section class="output-panel">
        <div class="panel-head">
          <h2>Boundary</h2>
          <span class="quiet">Explicit</span>
        </div>
        <ul class="boundary-list">
          ${scenarioManifest.expectations
            .map((expectation) => `<li><span>${expectation.label}</span><strong>${expectation.value}</strong></li>`)
            .join("")}
        </ul>
      </section>
    </aside>
  `;
}
