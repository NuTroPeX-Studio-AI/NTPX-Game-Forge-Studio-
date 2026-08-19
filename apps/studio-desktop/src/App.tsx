const stages = [
  "Repository Foundation",
  "GameSpec",
  "Autonomy Governor",
  "Model Router",
  "Agent Orchestrator",
  "Code Intelligence"
];

export default function App() {
  return (
    <main className="shell">
      <section className="hero">
        <p className="eyebrow">NTPX GAMEFORGE Ω∞</p>
        <h1>Autonomous Game Development Command Center</h1>
        <p className="lede">
          Phase 05 Agent Orchestrator is the B000014 verification candidate. Phase 06 Code Intelligence unlocks after exact-head CI.
        </p>
      </section>

      <section className="panel">
        <div><span className="label">Current build</span><strong>B000014</strong></div>
        <div><span className="label">Milestone</span><strong>M05</strong></div>
        <div><span className="label">State</span><strong>Candidate</strong></div>
      </section>

      <section className="roadmap">
        <h2>Roadmap</h2>
        <ol>
          {stages.map((stage, index) => (
            <li key={stage} className={index <= 4 ? "active" : "pending"}>
              <span>{String(index + 1).padStart(2, "0")}</span>
              {stage}
            </li>
          ))}
        </ol>
      </section>
    </main>
  );
}
