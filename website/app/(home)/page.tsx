import Link from 'next/link';
import { 
  Terminal, 
  Activity, 
  Rocket, 
  Cpu, 
  Github,
  Download,
  Eye,
  Keyboard,
  RefreshCw
} from 'lucide-react';
import { buttonVariants } from 'fumadocs-ui/components/ui/button';
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
          Mato is a terminal multiplexer and workspace that brings visual intelligence to the CLI.
          Manage hundreds of AI agent sessions with real-time signals, daemon-backed persistence, and zero-conflict ergonomics.
        </p>

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
          <h2 className="mb-6 text-3xl font-bold tracking-tight sm:text-4xl md:text-5xl">Why Mato?</h2>
          <p className="mx-auto max-w-2xl text-lg text-fd-muted-foreground">
            Traditional terminal multiplexers are "blind." You never know what's happening in another tab until you switch to it.
          </p>
        </div>

        <div className="grid gap-8 md:grid-cols-2 lg:grid-cols-4">
          {[
            {
              title: "Lost in Tabs",
              solution: "See active agents instantly with live activity signals across desks/tabs.",
              icon: <Eye className="h-6 w-6 text-[#D63C3C]" />
            },
            {
              title: "Arrow-Key Grind",
              solution: "Jump to what you need in one move, instead of stepping tab-by-tab.",
              icon: <Activity className="h-6 w-6 text-[#D63C3C]" />
            },
            {
              title: "Shortcut Hell",
              solution: "Only Esc is special, so your shell/editor shortcuts keep working as usual.",
              icon: <Keyboard className="h-6 w-6 text-[#D63C3C]" />
            },
            {
              title: "Session Loss",
              solution: "Agents keep running in the background; reconnect and continue where you left off.",
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

      {/* Features Grid */}
      <section className="container relative z-10 px-4 py-32 sm:px-6 md:py-48 bg-fd-secondary/10">
        <div className="mb-20 text-center">
          <h2 className="mb-6 text-3xl font-bold tracking-tight sm:text-4xl md:text-5xl">Features</h2>
          <p className="mx-auto max-w-2xl text-lg text-fd-muted-foreground text-balance">
            Developer-friendly features for high-concurrency AI workflows.
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
              title: "Jump Mode",
              description: "Press Esc, then jump straight to visible targets in one move.",
              img: "/screenshot-quick-jump-mode.gif"
            },
            {
              title: "Background Persistence",
              description: "Agents keep running even if terminal closes or SSH drops.",
              img: "/screenshot-daemon-background-run.gif"
            },
            {
              title: "Spinner Activity",
              description: "See where work is happening right now with live spinner signals.",
              img: "/screenshot-working-spinner.gif"
            },
            {
              title: "Mouse Support",
              description: "Click desks and tabs directly while keeping a terminal-native workflow.",
              img: "/screenshot-mouse-jump.gif"
            },
            {
              title: "Multi-Client Sync",
              description: "Attach from multiple terminals/SSH sessions and stay in sync.",
              img: "/screenshot-multi-client-sync.gif"
            },
            {
              title: "Prebuilt Templates",
              description: "Start in seconds with ready-made templates and multilingual onboarding.",
              img: "/screenshot-onboarding.png"
            },
            {
              title: "Customizable Themes",
              description: "Pick the theme style you like with persistent workspace state.",
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
            <p className="mt-6 text-sm font-medium text-white/85">
              Questions? Contact the author on{' '}
              <Link
                href="https://x.com/kellypeilinchan"
                target="_blank"
                rel="noreferrer"
                className="underline underline-offset-4 hover:text-white"
              >
                X
              </Link>
              .
            </p>
          </div>
          <div className="absolute -right-1/4 -top-1/4 h-[150%] w-[150%] rotate-12 bg-[radial-gradient(circle_at_center,rgba(255,255,255,0.15)_0%,transparent_70%)]" />
        </div>
      </section>

      {/* Pronunciation */}
      <section className="container relative z-10 px-4 pb-20 text-center sm:px-6 md:pb-24">
        <p className="text-sm font-mono italic text-fd-muted-foreground/70">
          You say tomato, I say Mato.
        </p>
        <p className="mt-2 text-xs text-fd-muted-foreground/60 sm:text-sm">
          English: MAY-to · 粤语: 咩圖 · 普通话: 梅拓 · 日本語: メイト · 한국어: 메이토
        </p>
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
