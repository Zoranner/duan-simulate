import "./styles.css";

document.querySelector<HTMLDivElement>("#app")!.innerHTML = `
  <main class="editor-shell" aria-label="DUAN editor">
    <aside class="sidebar" aria-label="Package and scenario">
      <div class="brand-row">
        <div>
          <p class="eyebrow">DUAN Editor</p>
          <h1>Authoring</h1>
        </div>
        <span class="mode-pill">Schema / Scenario</span>
      </div>

      <section class="panel">
        <div class="panel-head">
          <h2>Package</h2>
          <button type="button">Open</button>
        </div>
        <div class="tree-list">
          <button class="tree-item active" type="button">
            <span>warehouse-flow</span>
            <small>package.duan.json</small>
          </button>
          <button class="tree-item" type="button">
            <span>entities</span>
            <small>3 schemas</small>
          </button>
          <button class="tree-item" type="button">
            <span>components</span>
            <small>8 fields</small>
          </button>
        </div>
      </section>

      <section class="panel">
        <div class="panel-head">
          <h2>Scenario</h2>
          <button type="button">New</button>
        </div>
        <div class="tree-list">
          <button class="tree-item active" type="button">
            <span>morning-shift</span>
            <small>scenario.duan.json</small>
          </button>
          <button class="tree-item" type="button">
            <span>events</span>
            <small>12 queued</small>
          </button>
        </div>
      </section>
    </aside>

    <section class="workspace" aria-label="Entity and component authoring">
      <header class="toolbar">
        <div>
          <p class="eyebrow">Entity schema</p>
          <h2>Forklift</h2>
        </div>
        <div class="toolbar-actions">
          <button type="button">Validate</button>
          <button class="primary" type="button">Save</button>
        </div>
      </header>

      <form class="form-grid">
        <label>
          <span>Entity key</span>
          <input value="forklift" />
        </label>
        <label>
          <span>Display name</span>
          <input value="Forklift" />
        </label>
        <label>
          <span>Initial count</span>
          <input type="number" value="24" />
        </label>
        <label>
          <span>Scenario group</span>
          <select>
            <option>morning-shift</option>
            <option>night-shift</option>
          </select>
        </label>
      </form>

      <section class="component-panel">
        <div class="panel-head">
          <h2>Components</h2>
          <span class="quiet">Data only</span>
        </div>
        <div class="component-table" role="table" aria-label="Component fields">
          <div class="table-row table-head" role="row">
            <span role="columnheader">Name</span>
            <span role="columnheader">Type</span>
            <span role="columnheader">Default</span>
          </div>
          <div class="table-row" role="row">
            <span>position</span>
            <span>Vec2</span>
            <span>[0, 0]</span>
          </div>
          <div class="table-row" role="row">
            <span>battery</span>
            <span>f32</span>
            <span>100</span>
          </div>
          <div class="table-row" role="row">
            <span>task_state</span>
            <span>enum</span>
            <span>idle</span>
          </div>
        </div>
      </section>

      <section class="algorithm-note" aria-label="Algorithm boundary">
        <strong>Algorithm locked</strong>
        <span>Runtime systems are built in DUAN core packages.</span>
      </section>
    </section>

    <aside class="run-panel" aria-label="Run output">
      <header class="toolbar compact">
        <div>
          <p class="eyebrow">Run</p>
          <h2>Build</h2>
        </div>
        <button type="button" disabled>Pending</button>
      </header>

      <section class="metric-grid">
        <div>
          <span>Entities</span>
          <strong>24</strong>
        </div>
        <div>
          <span>Runner</span>
          <strong>Not built</strong>
        </div>
      </section>

      <section class="output-panel">
        <div class="panel-head">
          <h2>Output</h2>
          <span class="status-dot">Pending</span>
        </div>
        <pre>{
  "scenario": "morning-shift",
  "schema": "valid",
  "runner": "not-generated"
}</pre>
      </section>

      <section class="output-panel">
        <div class="panel-head">
          <h2>Events</h2>
          <span class="quiet">Latest</span>
        </div>
        <ol class="event-list">
          <li>package loaded</li>
          <li>scenario validated</li>
          <li>build/run connector pending</li>
        </ol>
      </section>
    </aside>
  </main>
`;
