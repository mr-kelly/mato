'use client';

import { useMemo, useState } from 'react';
import Link from 'next/link';

type InstallMethod = 'shell' | 'brew';

const SHELL_COMMAND = 'curl -fsSL https://raw.githubusercontent.com/mr-kelly/mato/main/install.sh | bash';
const BREW_COMMAND = 'brew tap mr-kelly/tap\nbrew install mato';

export function InstallTabs() {
  const [method, setMethod] = useState<InstallMethod>('shell');

  const command = useMemo(() => {
    return method === 'shell' ? SHELL_COMMAND : BREW_COMMAND;
  }, [method]);

  return (
    <section className="w-full max-w-3xl rounded-2xl border border-fd-border bg-fd-card p-4 text-left shadow-sm md:p-5">
      <div className="mb-3 flex flex-wrap items-center gap-2">
        <button
          type="button"
          onClick={() => setMethod('shell')}
          className={`rounded-md px-3 py-1.5 text-sm font-medium transition ${
            method === 'shell'
              ? 'bg-fd-primary text-fd-primary-foreground'
              : 'bg-fd-muted text-fd-muted-foreground hover:bg-fd-accent hover:text-fd-accent-foreground'
          }`}
        >
          Shell
        </button>
        <button
          type="button"
          onClick={() => setMethod('brew')}
          className={`rounded-md px-3 py-1.5 text-sm font-medium transition ${
            method === 'brew'
              ? 'bg-fd-primary text-fd-primary-foreground'
              : 'bg-fd-muted text-fd-muted-foreground hover:bg-fd-accent hover:text-fd-accent-foreground'
          }`}
        >
          Homebrew
        </button>
      </div>

      <pre className="overflow-x-auto rounded-lg bg-black px-4 py-3 text-sm leading-6 text-zinc-100">
        <code>{command}</code>
      </pre>

      {method === 'brew' ? (
        <p className="mt-3 text-sm text-fd-muted-foreground">
          Homebrew not installed?{' '}
          <Link
            className="font-medium text-fd-primary underline underline-offset-4"
            href="https://brew.sh/"
            target="_blank"
            rel="noreferrer"
          >
            Install Homebrew first
          </Link>
          .
        </p>
      ) : (
        <p className="mt-3 text-sm text-fd-muted-foreground">
          Works on Linux/macOS. Installs the latest stable Mato binary.
        </p>
      )}
    </section>
  );
}
