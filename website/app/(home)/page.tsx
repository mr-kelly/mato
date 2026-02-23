import Link from 'next/link';
import { 
  Terminal, 
  Activity, 
  Zap, 
  Layers, 
  Command, 
  Rocket, 
  ShieldCheck, 
  Cpu, 
  MousePointer2,
  Github,
  Download
} from 'lucide-react';
import { Card, Cards } from 'fumadocs-ui/components/card';
import { buttonVariants } from 'fumadocs-ui/components/ui/button';
import { Callout } from 'fumadocs-ui/components/callout';
import { InstallTabs } from '@/components/home/install-tabs';
import { VersionBadge } from '@/components/home/version-badge';
import { cn } from '@/lib/cn';

export default function HomePage() {
  const GITHUB_URL = "https://github.com/mr-kelly/mato";

  return (
    <main className="relative flex flex-col items-center overflow-x-hidden">
      {/* Hero Section */}
      <section className="relative z-10 flex flex-col items-center px-4 pt-12 pb-24 text-center sm:px-6 md:pt-20 md:pb-32">
        <div className="mb-6 flex items-center gap-2 rounded-full border border-fd-border bg-fd-secondary/30 px-4 py-1.5 text-sm font-medium backdrop-blur-md">
          <span className="relative flex h-2 w-2">
            <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-fd-primary opacity-75" />
            <span className="relative inline-flex h-2 w-2 rounded-full bg-fd-primary" />
          </span>
          <VersionBadge />
        </div>

        <h1 className="mb-6 max-w-5xl bg-gradient-to-b from-fd-foreground to-fd-foreground/70 bg-clip-text text-5xl font-extrabold tracking-tight text-transparent sm:text-6xl md:text-8xl">
          Your Agents, <br />
          <span className="text-fd-primary">Orchestrated.</span>
        </h1>
        
        <p className="mb-10 max-w-2xl text-balance text-lg leading-relaxed text-fd-muted-foreground md:text-xl px-4">
          Manage hundreds of AI agent sessions with real-time activity signals, 
          daemon-backed persistence, and zero-conflict ergonomics.
        </p>

        <div className="flex flex-wrap items-center justify-center gap-4 px-4">
          <Link 
            href={GITHUB_URL}
            target="_blank"
            rel="noreferrer"
            className={cn(buttonVariants({ color: 'primary' }), "w-full sm:w-auto rounded-full px-10 text-base font-bold transition-all hover:shadow-[0_0_20px_rgba(var(--fd-primary-rgb),0.3)]")}
          >
            Get Started
            <Github className="ml-2 h-4 w-4" />
          </Link>
          <Link
            href="#install"
            className={cn(buttonVariants({ color: 'outline' }), "w-full sm:w-auto rounded-full px-10 text-base font-medium transition-all")}
          >
            <Download className="mr-2 h-4 w-4 opacity-70" />
            Install
          </Link>
        </div>

        {/* Hero Visual */}
        <div className="mt-12 w-full max-w-5xl px-4 lg:mt-16">
          <div className="relative rounded-2xl border border-fd-border bg-fd-card/30 p-1.5 shadow-2xl backdrop-blur-sm sm:p-3 sm:rounded-3xl">
            <div className="absolute inset-0 -z-10 bg-gradient-to-tr from-fd-primary/10 via-transparent to-fd-primary/5 blur-2xl" />
            <img 
              src="/screenshot-office.png" 
              alt="Mato Terminal Office" 
              className="rounded-xl border border-fd-border/50 shadow-sm sm:rounded-2xl"
            />
          </div>
        </div>
      </section>

      {/* Quick Install */}
      <section id="install" className="container relative z-10 px-4 pb-32 sm:px-6 md:pb-48 scroll-mt-24">
        <div className="mx-auto max-w-4xl text-center mb-10">
          <div className="mb-6 flex flex-wrap justify-center gap-3">
            <div className="flex items-center gap-2 rounded-full border border-fd-border bg-fd-secondary/10 px-3 py-1 text-xs font-medium text-fd-muted-foreground">
              <Terminal className="h-3 w-3" />
              Linux & macOS
            </div>
            <div className="flex items-center gap-2 rounded-full border border-fd-border bg-fd-secondary/10 px-3 py-1 text-xs font-medium text-fd-muted-foreground">
              <Cpu className="h-3 w-3" />
              Agent Context Optimized
            </div>
          </div>
          
          <h2 className="mb-4 text-3xl font-bold tracking-tight sm:text-4xl">Ready in 60 seconds.</h2>
          <p className="mx-auto max-w-2xl text-fd-muted-foreground">
            Lightweight binary installation or AI-agent ready prompts.
          </p>
        </div>
        
        <div className="relative mx-auto max-w-4xl">
          <div className="absolute inset-0 -z-10 bg-fd-primary/5 blur-3xl opacity-50" />
          <div className="rounded-2xl border border-fd-border bg-fd-card/80 p-2 shadow-2xl backdrop-blur-md sm:p-6 md:rounded-3xl">
            <InstallTabs />
          </div>
        </div>
      </section>

      {/* Feature Pillars */}
      <section className="container relative z-10 px-4 py-32 sm:px-6 md:py-48">
        <div className="mb-20 text-center">
          <h2 className="mb-6 text-3xl font-bold tracking-tight sm:text-4xl md:text-5xl">Built for the AI Era</h2>
          <p className="mx-auto max-w-2xl text-lg text-fd-muted-foreground">
            A radical rethink of terminal multiplexing, designed for the high-concurrency 
            workflows of modern AI-driven development.
          </p>
        </div>

        <Cards className="grid sm:grid-cols-2 lg:grid-cols-4 border-none shadow-none bg-transparent gap-4">
          <Card
            icon={<Activity className="h-5 w-5 text-fd-primary" />}
            title="Live Signals"
            description="Visual indicators show output activity across all desks and tabs instantly."
          />
          <Card
            icon={<Zap className="h-5 w-5 text-fd-primary" />}
            title="Zero Conflicts"
            description="The 'Rule of One' ensures Mato stays out of your shell and editor's way."
          />
          <Card
            icon={<Layers className="h-5 w-5 text-fd-primary" />}
            title="Office Model"
            description="Organize hundreds of agent sessions into structured Desks and Tabs."
          />
          <Card
            icon={<ShieldCheck className="h-5 w-5 text-fd-primary" />}
            title="Daemon Backend"
            description="State persists in a background daemon. Never lose your workspace again."
          />
        </Cards>
      </section>

      {/* Philosophy */}
      <section className="container relative z-10 px-4 py-32 sm:px-6 md:py-48">
        <div className="rounded-3xl border border-fd-border bg-fd-secondary/30 p-6 sm:p-10 md:p-20 md:rounded-[2.5rem]">
          <div className="mb-16 text-center">
            <h2 className="mb-6 text-3xl font-bold tracking-tight sm:text-4xl md:text-5xl">The Philosophy</h2>
            <p className="mx-auto max-w-2xl text-lg text-fd-muted-foreground">
              Traditional terminal multiplexers (tmux/screen) were built for older workflows.
              Mato is built for modern human-agent orchestration.
            </p>
          </div>

          <div className="grid gap-12 md:grid-cols-2">
            <div className="space-y-6">
              <h3 className="flex items-center gap-3 text-2xl font-bold">
                <MousePointer2 className="h-6 w-6 text-fd-muted-foreground" />
                Legacy Tools
              </h3>
              <ul className="space-y-4">
                {[
                  "Mental tax of prefix keys (Ctrl-b/a) slows down flow.",
                  "Zero visibility into background tab output.",
                  "Scale breaks quickly with dozens of concurrent agents."
                ].map((text, i) => (
                  <li key={i} className="flex items-start gap-3 italic text-fd-muted-foreground/80">
                    <span className="text-fd-muted-foreground/40">•</span>
                    <span>{text}</span>
                  </li>
                ))}
              </ul>
            </div>
            
            <div className="space-y-6">
              <h3 className="flex items-center gap-3 text-2xl font-bold">
                <Command className="h-6 w-6 text-fd-primary" />
                The Mato Way
              </h3>
              <ul className="space-y-4">
                {[
                  "Instant visual signals for all active processes.",
                  "Jump Mode: One key to rule them all (Esc).",
                  "Built-in Office/Desk hierarchy for infinite scale."
                ].map((text, i) => (
                  <li key={i} className="flex items-start gap-3 font-medium">
                    <span className="text-fd-primary font-bold">✓</span>
                    <span>{text}</span>
                  </li>
                ))}
              </ul>
            </div>
          </div>
          
          <div className="mt-16">
            <Callout type="warn" title="Rule of One">
              Mato reserves ONLY the `Esc` key for system-level navigation. This ensures 100% compatibility with 
              your Neovim configs and shell tools. No more shortcut wrestling.
            </Callout>
          </div>
        </div>
      </section>

      {/* Shortcuts */}
      <section className="container relative z-10 px-4 pb-32 sm:px-6 md:pb-48">
        <div className="mb-16 text-center">
          <h2 className="mb-6 text-3xl font-bold tracking-tight sm:text-4xl md:text-5xl">Master the Flow</h2>
          <p className="text-lg text-fd-muted-foreground text-balance max-w-2xl mx-auto">
            A minimalist shortcut set designed for muscle memory. 
            Once in Jump Mode, the terminal is your playground.
          </p>
        </div>

        <div className="mx-auto max-w-4xl overflow-x-auto rounded-2xl border border-fd-border bg-fd-card shadow-2xl md:rounded-[2rem]">
          <table className="w-full min-w-[600px] border-collapse text-left">
            <thead>
              <tr className="bg-fd-muted/50 border-b border-fd-border">
                <th className="px-8 py-5 font-bold uppercase tracking-wider text-xs text-fd-muted-foreground">Key</th>
                <th className="px-8 py-5 font-bold uppercase tracking-wider text-xs text-fd-muted-foreground">Action</th>
                <th className="px-8 py-5 font-bold uppercase tracking-wider text-xs text-fd-muted-foreground text-right">Scope</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-fd-border/50">
              {[
                ['Esc', 'Jump / Teleport Mode', 'Global'],
                ['n', 'Create New Context', 'Navigation'],
                ['x', 'Terminate Session', 'Navigation'],
                ['r', 'Rename Context', 'Navigation'],
                ['o', 'Office Selector', 'Global'],
                ['q', 'Soft Quit Client', 'Global'],
              ].map(([key, action, scope]) => (
                <tr key={key} className="group hover:bg-fd-primary/[0.02] transition-colors">
                  <td className="px-8 py-5">
                    <kbd className="rounded-lg border border-fd-border bg-fd-muted px-2.5 py-1.5 font-mono text-[13px] font-bold shadow-sm transition-transform group-hover:scale-110 inline-block">
                      {key}
                    </kbd>
                  </td>
                  <td className="px-8 py-5 font-semibold text-fd-foreground/90">{action}</td>
                  <td className="px-8 py-5 text-fd-muted-foreground text-right font-medium">{scope}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* CTA Section */}
      <section className="container relative z-10 px-4 py-32 text-center sm:px-6 md:py-48">
        <div className="relative overflow-hidden rounded-3xl bg-fd-primary px-6 py-16 text-fd-primary-foreground shadow-[0_20px_50px_rgba(var(--fd-primary-rgb),0.3)] md:rounded-[3rem] md:px-8 md:py-20">
          <div className="relative z-10 mx-auto max-w-3xl">
            <h2 className="mb-8 text-3xl font-extrabold tracking-tight sm:text-4xl md:text-6xl">Scale your productivity.</h2>
            <p className="mb-12 text-lg font-medium opacity-90 md:text-xl">Join the next generation of terminal orchestration.</p>
            <div className="flex flex-col items-center gap-4 sm:flex-row sm:justify-center">
              <Link 
                href={GITHUB_URL}
                target="_blank"
                rel="noreferrer"
                className={cn(buttonVariants({ color: 'secondary' }), "w-full rounded-full px-14 py-7 text-xl font-bold transition-all hover:scale-105 active:scale-95 sm:w-auto")}
              >
                Start Building Now
                <Rocket className="ml-3 h-6 w-6" />
              </Link>
            </div>
          </div>
          {/* Decorative gradients */}
          <div className="absolute -right-1/4 -top-1/4 h-[150%] w-[150%] rotate-12 bg-[radial-gradient(circle_at_center,rgba(255,255,255,0.15)_0%,transparent_70%)]" />
        </div>
      </section>

      {/* Decorative Background Elements */}
      <div className="fixed inset-0 -z-10 overflow-hidden pointer-events-none">
        <div className="absolute -top-[10%] left-[10%] h-[40%] w-[40%] rounded-full bg-fd-primary/5 blur-[120px]" />
        <div className="absolute bottom-[10%] right-[10%] h-[40%] w-[40%] rounded-full bg-fd-primary/5 blur-[120px]" />
        <div className="absolute inset-0 bg-[linear-gradient(to_right,#80808008_1px,transparent_1px),linear-gradient(to_bottom,#80808008_1px,transparent_1px)] bg-[size:64px_64px] [mask-image:radial-gradient(ellipse_80%_80%_at_50%_0%,#000_60%,transparent_100%)]" />
      </div>
    </main>
  );
}
