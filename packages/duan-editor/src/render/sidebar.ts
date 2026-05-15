import type { EditorFixture } from "../data/loadFixture";

export function renderSidebar(fixture: EditorFixture): string {
  const { packageMetadata, scenarioManifest, boundary } = fixture;

  return `
    <aside class="sidebar" aria-label="Fixture package and scenario">
      <div class="brand-row">
        <div>
          <p class="eyebrow">DUAN Editor</p>
          <h1>${packageMetadata.title}</h1>
        </div>
        <span class="mode-pill">Read-only</span>
      </div>

      <section class="notice-panel" aria-label="Editor boundary">
        <strong>Fixture browsing only</strong>
        <span>CLI ${boundary.cli}; runner ${boundary.runner}.</span>
      </section>

      <section class="panel">
        <div class="panel-head">
          <h2>Package</h2>
          <span class="quiet">Open pending</span>
        </div>
        <div class="tree-list">
          <button class="tree-item active" type="button" disabled>
            <span>${packageMetadata.name}</span>
            <small>${packageMetadata.version} / ${boundary.source}</small>
          </button>
          <button class="tree-item" type="button" disabled>
            <span>components</span>
            <small>${packageMetadata.components.length} definitions</small>
          </button>
          <button class="tree-item" type="button" disabled>
            <span>systems</span>
            <small>${packageMetadata.systems.length} fixed-step systems</small>
          </button>
        </div>
      </section>

      <section class="panel">
        <div class="panel-head">
          <h2>Scenario</h2>
          <span class="quiet">New pending</span>
        </div>
        <div class="tree-list">
          <button class="tree-item active" type="button" disabled>
            <span>${scenarioManifest.name}</span>
            <small>${scenarioManifest.id}</small>
          </button>
          <button class="tree-item" type="button" disabled>
            <span>entities</span>
            <small>${scenarioManifest.entities.length} fixture entity</small>
          </button>
          <button class="tree-item" type="button" disabled>
            <span>events</span>
            <small>${scenarioManifest.events.length} read-only notes</small>
          </button>
        </div>
      </section>
    </aside>
  `;
}
