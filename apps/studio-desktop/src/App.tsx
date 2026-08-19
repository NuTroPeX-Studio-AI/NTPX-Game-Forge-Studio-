const stages = [
  "Repository Foundation",
  "GameSpec",
  "Autonomy Governor",
  "Model Router",
  "Agent Orchestrator",
  "Code Intelligence",
  "Agent Zero Execution"
];

export default function App() {
  return (
    <main className="shell">
      <section className="hero">
        <p className="eyebrow">NTPX GAMEFORGE Ω∞</p>
        <h1>Autonomous Game Development Command Center</h1>
        <p className="lede">
          Phase 06 Code Intelligence is a B000028 verification candidate. Phase 07 remains gated until exact-head CI and Fortress verification pass.
        </p>
      </section>

      <section className="panel">
        <div><span className="label">Current build</span><strong>B000028</strong></div>
        <div><span className="label">Milestone</span><strong>M06</strong></div>
        <div><span className="label">State</span><strong>Candidate</strong></div>
      </section>

      <section className="roadmap">
        <h2>Roadmap</h2>
        <ol>
          {stages.map((stage, index) => (
            <li key={stage} className={index <= 5 ? "active" : "pending"}>
              <span>{String(index + 1).padStart(2, "0")}</span>
              {stage}
            </li>
          ))}
        </ol>
      </section>
    </main>
  );
}
