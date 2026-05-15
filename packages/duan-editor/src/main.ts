import { loadFixture } from "./data/loadFixture";
import { renderSidebar } from "./render/sidebar";
import { renderStatusPanel } from "./render/statusPanel";
import { renderWorkspace } from "./render/workspace";
import "./styles.css";

const fixture = loadFixture();

document.querySelector<HTMLDivElement>("#app")!.innerHTML = `
  <main class="editor-shell" aria-label="DUAN read-only fixture editor">
    ${renderSidebar(fixture)}
    ${renderWorkspace(fixture)}
    ${renderStatusPanel(fixture)}
  </main>
`;
