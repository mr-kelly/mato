import Link from 'next/link';
import { InstallTabs } from '@/components/home/install-tabs';

export default function HomePage() {
  const signalCards = [
    {
      problem: 'Lost in tab hunting',
      solution: 'Live activity indicators show where output is happening right now.',
    },
    {
      problem: 'Shortcut conflicts everywhere',
      solution: "Rule of One: only Esc is special. Your shell shortcuts stay untouched.",
    },
    {
      problem: 'Agent sprawl becomes chaos',
      solution: 'Desk + Tab office model keeps hundreds of sessions structured.',
    },
    {
      problem: 'Client crash means panic',
      solution: 'Daemon-first persistence keeps workspaces alive across reconnects.',
    },
  ];

  const featurePillars = [
    {
      title: 'Live Activity Monitoring',
      text: 'Spot active builds and agent output instantly with sidebar/topbar signals.',
    },
    {
      title: 'AI-Agent Native',
      text: 'Built for Claude Code, Cursor, and Windsurf with zero shell shortcut conflicts.',
    },
    {
      title: 'Jump Mode Teleport',
      text: 'Esc-driven target labels let you jump across desks and tabs in one move.',
    },
    {
      title: 'Office Templates',
      text: 'Start from curated setups and scale from solo coding to high-agent orchestration.',
    },
  ];

  const shortcutRows = [
    ['Esc', 'Jump / Teleport', 'Global'],
    ['n', 'Create New', 'Sidebar/Topbar'],
    ['x', 'Close / Terminate', 'Sidebar/Topbar'],
    ['r', 'Rename', 'Sidebar/Topbar'],
    ['o', 'Office Selector', 'Sidebar'],
    ['q', 'Soft Quit', 'Sidebar'],
  ];

  return (
    <main className="mato-home">
      <section className="mato-hero reveal-1">
        <div className="mato-badge">Multi-Agent Terminal Office</div>
        <img className="mato-hero-logo" src="/logo.svg" alt="Mato logo" width={104} height={104} />
        <h1>Managing hundreds of AI agents from the command line.</h1>
        <p>
          Mato turns terminal multiplexing into visual operations: desks, tabs, activity signals, and
          daemon-backed persistence, designed for AI-native workflows.
        </p>
        <div className="mato-hero-ctas">
          <Link href="/docs" className="mato-btn mato-btn-primary">
            Read Docs
          </Link>
          <Link href="https://github.com/mr-kelly/mato" target="_blank" rel="noreferrer" className="mato-btn">
            View on GitHub
          </Link>
        </div>
      </section>

      <section className="mato-showcase reveal-2">
        <div className="mato-section-head">
          <h2>Showcase</h2>
          <p>One office, many concurrent flows. Keep situational awareness without tab roulette.</p>
        </div>
        <div className="mato-shot-wrap">
          <img src="/screenshot-0.png" alt="Mato terminal office screenshot" className="mato-shot" />
        </div>
      </section>

      <section className="mato-signal-grid reveal-3">
        {signalCards.map((card) => (
          <article key={card.problem} className="mato-card">
            <h3>{card.problem}</h3>
            <p>{card.solution}</p>
          </article>
        ))}
      </section>

      <section className="mato-compare reveal-4">
        <div className="mato-section-head">
          <h2>Why Mato</h2>
          <p>tmux is powerful. Mato adds visual intelligence and AI-agent-native ergonomics.</p>
        </div>
        <div className="mato-compare-grid">
          <article className="mato-card">
            <h3>Traditional Multiplexers</h3>
            <ul>
              <li>Hard to see activity without switching tabs</li>
              <li>Prefix-heavy shortcuts can conflict with shell and agents</li>
              <li>Low visibility for large concurrent workflows</li>
            </ul>
          </article>
          <article className="mato-card">
            <h3>Mato Approach</h3>
            <ul>
              <li>Real-time activity signals in desk/tab navigation</li>
              <li>Rule of One: only Esc is reserved by the system</li>
              <li>Office model for high-scale parallel agent operations</li>
            </ul>
          </article>
        </div>
      </section>

      <section className="mato-feature-grid reveal-5">
        {featurePillars.map((feature) => (
          <article key={feature.title} className="mato-card mato-card-feature">
            <h3>{feature.title}</h3>
            <p>{feature.text}</p>
          </article>
        ))}
      </section>

      <section className="mato-framework reveal-5">
        <div>
          <h2>Shortcut Philosophy: Rule of One</h2>
          <p>
            Mato reserves one key for system navigation. Everything else stays available to your shell,
            editor, and AI tools.
          </p>
        </div>
        <div className="mato-shortcuts">
          <table>
            <thead>
              <tr>
                <th>Key</th>
                <th>Action</th>
                <th>Context</th>
              </tr>
            </thead>
            <tbody>
              {shortcutRows.map((row) => (
                <tr key={row[0]}>
                  <td>
                    <code>{row[0]}</code>
                  </td>
                  <td>{row[1]}</td>
                  <td>{row[2]}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      <section className="mato-install reveal-5">
        <div>
          <h2>Get started in 60 seconds.</h2>
          <p>Install Mato, launch your first office, choose a template, and start shipping.</p>
        </div>
        <InstallTabs />
      </section>
    </main>
  );
}
