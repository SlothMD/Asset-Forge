import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";

const sampleRows = [
  {
    id: "card_001",
    title: "Patch the Hull",
    type: "Action",
    cost: 2,
    rulesText: "Repair 2 hull damage.",
    faction: "Freeholders"
  },
  {
    id: "card_002",
    title: "Lock Target",
    type: "Tactic",
    cost: 1,
    rulesText: "Gain target lock on one enemy ship.",
    faction: "Freeholders"
  },
  {
    id: "card_003",
    title: "Burn Hard",
    type: "Action",
    cost: 3,
    rulesText: "Move one extra sector and gain 1 heat.",
    faction: "PACS"
  }
];

function App() {
  return (
    <main className="app-shell">
      <section className="workspace">
        <div className="toolbar">
          <div>
            <p className="eyebrow">Asset Forge</p>
            <h1>Project Hub</h1>
          </div>
          <div className="actions">
            <button type="button">Open Project</button>
            <button type="button">Import CSV</button>
            <button type="button" className="primary">
              Export Build
            </button>
          </div>
        </div>

        <div className="content-grid">
          <aside className="panel">
            <h2>POC Pipeline</h2>
            <ol>
              <li>Create local project folder</li>
              <li>Import CSV card data</li>
              <li>Preview rows</li>
              <li>Render SVG card previews</li>
              <li>Export PNG and PDF</li>
            </ol>
          </aside>

          <section className="table-panel">
            <div className="section-heading">
              <h2>Sample Card Data</h2>
              <span>{sampleRows.length} rows</span>
            </div>
            <table>
              <thead>
                <tr>
                  <th>Title</th>
                  <th>Faction</th>
                  <th>Type</th>
                  <th>Cost</th>
                </tr>
              </thead>
              <tbody>
                {sampleRows.map((row) => (
                  <tr key={row.id}>
                    <td>{row.title}</td>
                    <td>{row.faction}</td>
                    <td>{row.type}</td>
                    <td>{row.cost}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </section>

          <section className="preview-panel">
            <div className="section-heading">
              <h2>Template Preview</h2>
              <span>2.5 x 3.5 in</span>
            </div>
            <article className="card-preview">
              <div className="cost-badge">2</div>
              <div className="faction-badge">Freeholders</div>
              <h3>Patch the Hull</h3>
              <div className="art-slot">Art</div>
              <p>Repair 2 hull damage.</p>
              <footer>Action</footer>
            </article>
          </section>
        </div>
      </section>
    </main>
  );
}

createRoot(document.getElementById("root") as HTMLElement).render(
  <StrictMode>
    <App />
  </StrictMode>
);
