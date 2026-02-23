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
  Download,
  Eye,
  Keyboard,
  RefreshCw,
  LayoutGrid,
  Palette
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
            <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-[#D63C3C] opacity-75" />
            <span className="relative inline-flex h-2 w-2 rounded-full bg-[#D63C3C]" />
          </span>
          <VersionBadge />
        </div>

        <h1 className="mb-6 max-w-5xl bg-gradient-to-b from-fd-foreground to-fd-foreground/70 bg-clip-text text-5xl font-extrabold tracking-tight text-transparent sm:text-6xl md:text-8xl">
          Your Agents, <br />
          <span className="text-[#D63C3C]">Orchestrated.</span>
        </h1>
        
        <p className="mb-4 max-w-2xl text-balance text-lg leading-relaxed text-fd-muted-foreground md:text-xl px-4">
          The <strong>Tomato Terminal</strong> for managing hundreds of AI agent sessions with real-time activity signals, 
          daemon-backed persistence, and zero-conflict ergonomics.
        </p>

        <div className="mb-10 flex flex-col items-center gap-1 text-sm font-mono text-fd-muted-foreground/60 italic">
          <p>You say tomato, I say Mato.</p>
          <div className="flex gap-4 opacity-50 text-[10px] sm:text-sm">
            <span>English: MAY-to</span>
            <span>粤语: 咩圖</span>
            <span>普通话: 梅拓</span>
            <span>Spanish: undefined</span>
          </div>
        </div>

        <div className="flex flex-wrap items-center justify-center gap-4 px-4">
          <Link 
            href={GITHUB_URL}
            target="_blank"
            rel="noreferrer"
            className={cn(buttonVariants({ color: 'primary' }), "w-full sm:w-auto rounded-full px-10 text-base font-bold bg-[#D63C3C] hover:bg-[#B02A2A] text-white transition-all hover:shadow-[0_0_20px_rgba(214,60,60,0.3)]")}
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
            <div className="absolute inset-0 -z-10 bg-gradient-to-tr from-[#D63C3C]/10 via-transparent to-[#D63C3C]/5 blur-2xl" />
            <img 
              src="/screenshot-coding.png" 
              alt="Mato Terminal Office" 
              className="rounded-xl border border-fd-border/50 shadow-sm sm:rounded-2xl"
            />
          </div>
        </div>
      </section>

      {/* The Vision Section */}
      <section className="container relative z-10 px-4 py-32 sm:px-6 md:py-48">
        <div className="mb-20 text-center">
          <h2 className="mb-6 text-3xl font-bold tracking-tight sm:text-4xl md:text-5xl">Visual Intelligence for CLI</h2>
          <p className="mx-auto max-w-2xl text-lg text-fd-muted-foreground">
            Traditional terminal multiplexers are "blind." Mato changes that by bringing 
            real-time visibility to your command line.
          </p>
        </div>

        <div className="grid gap-8 md:grid-cols-2 lg:grid-cols-4">
          {[
            {
              title: "Lost in Tabs",
              solution: "Real-time Activity Spinners notify you exactly where the work is happening.",
              icon: <Eye className="h-6 w-6 text-[#D63C3C]" />
            },
            {
              title: "Shortcut Hell",
              solution: "Zero-Conflict Design: Your shell belongs to you. Only Esc is special.",
              icon: <Keyboard className="h-6 w-6 text-[#D63C3C]" />
            },
            {
              title: "Task Anxiety",
              solution: "Visual Breadcrumbs: Instant status of every background agent or build process.",
              icon: <Activity className="h-6 w-6 text-[#D63C3C]" />
            },
            {
              title: "Session Loss",
              solution: "Daemon-First Architecture: Your workspace lives even if the client dies.",
              icon: <RefreshCw className="h-6 w-6 text-[#D63C3C]" />
            }
          ].map((item, i) => (
            <div key={i} className="flex flex-col gap-4 rounded-2xl border border-fd-border bg-fd-card/50 p-6">
              <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-[#D63C3C]/10">
                {item.icon}
              </div>
              <h3 className="text-xl font-bold">{item.title}</h3>
              <p className="text-sm leading-relaxed text-fd-muted-foreground">{item.solution}</p>
            </div>
          ))}
        </div>
      </section>

      {/* Showcase Grid */}
      <section className="container relative z-10 px-4 py-32 sm:px-6 md:py-48 bg-fd-secondary/10">
        <div className="mb-20 text-center">
          <h2 className="mb-6 text-3xl font-bold tracking-tight sm:text-4xl md:text-5xl">Workspace Showcase</h2>
          <p className="mx-auto max-w-2xl text-lg text-fd-muted-foreground text-balance">
            Everything you need for high-concurrency agent workflows.
          </p>
        </div>

        <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-2 xl:grid-cols-2">
          {[
            {
              title: "Offices -> Desks -> Tabs",
              description: "A structured workspace hierarchy for parallel terminal workflows.",
              img: "/screenshot-office.png"
            },
            {
              title: "Spinner Activity",
              description: "Live indicators appear the moment an agent or process produces output.",
              img: "/screenshot-working-spinner.gif"
            },
            {
              title: "Multi-Client Sync",
              description: "Attach multiple SSH clients to the same daemon with synchronized state.",
              img: "/screenshot-multi-client-sync.gif"
            },
            {
              title: "Prebuilt Templates",
              description: "Fast onboarding with curated layouts for different engineering roles.",
              img: "/screenshot-onboarding.png"
            },
            {
              title: "Background Persistence",
              description: "Sessions keep running even if your terminal process exits or SSH disconnects.",
              img: "/screenshot-daemon-background-run.gif"
            },
            {
              title: "Mouse Support",
              description: "Native mouse interaction for clicking tabs and desks in a terminal UI.",
              img: "/screenshot-mouse-jump.gif"
            },
            {
              title: "Jump Mode",
              description: "Press Esc to enter Jump Mode, then teleport instantly to visible targets.",
              img: "/screenshot-quick-jump-mode.gif"
            },
            {
              title: "Customizable Themes",
              description: "Personalize your experience with built-in themes like Tomato and Nord.",
              img: "/screenshot-themes.png"
            }
          ].map((item, i) => (
            <div key={i} className="flex flex-col overflow-hidden rounded-2xl border border-fd-border bg-fd-card">
              <div className="aspect-[16/10] overflow-hidden bg-black/40">
                <img src={item.img} alt={item.title} className="h-full w-full object-cover transition-transform hover:scale-105" />
              </div>
              <div className="p-6">
                <h3 className="mb-2 text-xl font-bold">{i + 1}. {item.title}</h3>
                <p className="text-sm text-fd-muted-foreground">{item.description}</p>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* Quick Install */}
      <section id="install" className="container relative z-10 px-4 pt-32 pb-32 sm:px-6 md:pb-48 scroll-mt-24">
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
          <div className="absolute inset-0 -z-10 bg-[#D63C3C]/5 blur-3xl opacity-50" />
          <div className="rounded-2xl border border-fd-border bg-fd-card/80 p-2 shadow-2xl backdrop-blur-md sm:p-6 md:rounded-3xl">
            <InstallTabs />
          </div>
        </div>
      </section>

      {/* Premium Features */}
      <section className="container relative z-10 px-4 py-32 sm:px-6 md:py-48">
        <div className="mb-20 text-center">
          <h2 className="mb-6 text-3xl font-bold tracking-tight sm:text-4xl md:text-5xl">Engineered for Clarity</h2>
        </div>

        <div className="grid gap-8 md:grid-cols-2">
          <div className="space-y-4 rounded-3xl border border-fd-border bg-fd-card/50 p-8">
            <Activity className="h-8 w-8 text-[#D63C3C]" />
            <h3 className="text-2xl font-bold">Live Activity Monitoring</h3>
            <p className="text-fd-muted-foreground">
              Never poll your terminals again. Mato's signature spinners appear in your sidebar and topbar the moment a process produces output. Perfect for tracking long-running builds or AI agents.
            </p>
          </div>
          <div className="space-y-4 rounded-3xl border border-fd-border bg-fd-card/50 p-8">
            <Command className="h-8 w-8 text-[#D63C3C]" />
            <h3 className="text-2xl font-bold">AI-Agent Native</h3>
            <p className="text-fd-muted-foreground">
              Built specifically for tools like Claude Code, Cursor, and Windsurf. Mato preserves 100% of your shell's keyboard shortcuts, ensuring your agents operate without interference.
            </p>
          </div>
          <div className="space-y-4 rounded-3xl border border-fd-border bg-fd-card/50 p-8">
            <Zap className="h-8 w-8 text-[#D63C3C]" />
            <h3 className="text-2xl font-bold">Jump Mode (EasyMotion)</h3>
            <p className="text-fd-muted-foreground">
              Navigate like a pro. Hit Esc and use EasyMotion-style jump labels to teleport to any desk or tab instantly. No more repetitive arrow-key mashing.
            </p>
          </div>
          <div className="space-y-4 rounded-3xl border border-fd-border bg-fd-card/50 p-8">
            <Palette className="h-8 w-8 text-[#D63C3C]" />
            <h3 className="text-2xl font-bold">Customizable Themes</h3>
            <p className="text-fd-muted-foreground">
              Personalize your workspace with a suite of professional themes including Tomato, Potato, Nord, and Darcula. Tailored for long-term coding comfort.
            </p>
          </div>
        </div>
      </section>

      {/* Shortcuts */}
      <section className="container relative z-10 px-4 pb-32 sm:px-6 md:pb-48">
        <div className="mb-16 text-center">
          <h2 className="mb-6 text-3xl font-bold tracking-tight sm:text-4xl md:text-5xl">Shortcut Philosophy</h2>
          <p className="text-lg text-fd-muted-foreground text-balance max-w-2xl mx-auto">
            The <strong>Rule of One</strong>: you don't need to memorize shortcuts. 
            Esc is the only state-switch key, and everything else stays with your shell.
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
                <tr key={key} className="group hover:bg-[#D63C3C]/[0.02] transition-colors">
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
        <div className="relative overflow-hidden rounded-3xl bg-[#D63C3C] px-6 py-16 text-white shadow-[0_20px_50px_rgba(214,60,60,0.3)] md:rounded-[3rem] md:px-8 md:py-20">
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
          <div className="absolute -right-1/4 -top-1/4 h-[150%] w-[150%] rotate-12 bg-[radial-gradient(circle_at_center,rgba(255,255,255,0.15)_0%,transparent_70%)]" />
        </div>
      </section>

      {/* Decorative Background Elements */}
      <div className="fixed inset-0 -z-10 overflow-hidden pointer-events-none">
        <div className="absolute -top-[10%] left-[10%] h-[40%] w-[40%] rounded-full bg-[#D63C3C]/5 blur-[120px]" />
        <div className="absolute bottom-[10%] right-[10%] h-[40%] w-[40%] rounded-full bg-[#D63C3C]/5 blur-[120px]" />
        <div className="absolute inset-0 bg-[linear-gradient(to_right,#80808008_1px,transparent_1px),linear-gradient(to_bottom,#80808008_1px,transparent_1px)] bg-[size:64px_64px] [mask-image:radial-gradient(ellipse_80%_80%_at_50%_0%,#000_60%,transparent_100%)]" />
      </div>
    </main>
  );
}
