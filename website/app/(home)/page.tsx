import Link from 'next/link';
import { InstallTabs } from '@/components/home/install-tabs';

export default function HomePage() {
  return (
    <div className="mx-auto flex w-full max-w-5xl flex-1 flex-col items-center px-4 pb-16 pt-10 text-center md:pt-16">
      <p className="mb-4 inline-flex rounded-full border border-fd-border bg-fd-secondary px-3 py-1 text-xs font-semibold uppercase tracking-wider text-fd-secondary-foreground">
        Multi-Agent Terminal Multiplexer
      </p>

      <h1 className="max-w-4xl text-balance text-4xl font-bold tracking-tight md:text-6xl">
        Mato keeps your terminal office organized while agents run in parallel.
      </h1>

      <p className="mt-5 max-w-3xl text-pretty text-base text-fd-muted-foreground md:text-lg">
        A modern terminal multiplexer with activity visibility, desk/tab workflows, and zero shortcut
        conflict for AI-native command line development.
      </p>

      <div className="mt-8 flex flex-wrap justify-center gap-3">
        <Link
          href="/docs"
          className="rounded-md bg-fd-primary px-4 py-2 text-sm font-semibold text-fd-primary-foreground transition hover:opacity-90"
        >
          Read Docs
        </Link>
        <Link
          href="https://github.com/mr-kelly/mato"
          target="_blank"
          rel="noreferrer"
          className="rounded-md border border-fd-border px-4 py-2 text-sm font-semibold text-fd-foreground transition hover:bg-fd-accent"
        >
          View on GitHub
        </Link>
      </div>

      <div className="mt-10 w-full">
        <InstallTabs />
      </div>
    </div>
  );
}
