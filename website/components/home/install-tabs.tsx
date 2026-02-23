'use client';

import Link from 'next/link';
import { Callout } from 'fumadocs-ui/components/callout';
import { CodeBlock, Pre } from 'fumadocs-ui/components/codeblock';
import { Tab, Tabs } from 'fumadocs-ui/components/tabs';

const HUMAN_QUICK_COMMAND = 'curl -fsSL http://mato.sh/install.sh | bash';
const HUMAN_BREW_COMMAND = 'brew tap mr-kelly/tap\nbrew install mato';

const AGENT_PROMPT = `Install Mato on this machine and verify it works.

Steps:
1) Primary install path (official install script):
   - curl -fsSL http://mato.sh/install.sh | bash
2) If that fails, fallback A (Homebrew):
   - brew tap mr-kelly/tap
   - brew install mato
3) If Homebrew is unavailable/fails, fallback B (GitHub release binary):
   - https://github.com/mr-kelly/mato/releases/latest
   - Extract and install "mato" into /usr/local/bin (or ~/.local/bin without sudo)
4) If binary install also fails, fallback C (build from source):
   - git clone https://github.com/mr-kelly/mato.git
   - cd mato
   - cargo build --release
   - install target/release/mato to /usr/local/bin (or ~/.local/bin)
5) Verification:
   - run: mato --version
   - launch mato once, confirm startup works, then exit
6) If any step fails, explain the exact failure and continue with the next fallback automatically.`;

export function InstallTabs() {
  return (
    <Tabs items={['For AI Agent', 'For Human']} className="w-full" defaultIndex={0}>
      <Tab>
        <div className="space-y-3 px-1 pt-2 sm:px-2">
          <p className="text-sm text-fd-muted-foreground">
            Copy this prompt into Claude Code, Codex, Warp, Cursor, Antigravity, OpenClaw,
            GitHub Copilot (VS Code), Gemini CLI, or Windsurf.
          </p>
          <CodeBlock title="Prompt" className="my-0">
            <Pre className="p-4 md:p-6 text-[13px] leading-relaxed">
              <code>{AGENT_PROMPT}</code>
            </Pre>
          </CodeBlock>
        </div>
      </Tab>
      <Tab>
        <div className="space-y-4 px-1 pt-2 sm:px-2">
          <CodeBlock title="Quick Install (Linux/macOS)" className="my-0">
            <Pre className="p-4 md:p-6 text-[13px]">
              <code>{HUMAN_QUICK_COMMAND}</code>
            </Pre>
          </CodeBlock>
          <CodeBlock title="Homebrew (Linux/macOS)" className="my-0">
            <Pre className="p-4 md:p-6 text-[13px]">
              <code>{HUMAN_BREW_COMMAND}</code>
            </Pre>
          </CodeBlock>
          <Callout type="info">
            Homebrew works on Linux and macOS.{' '}
            <Link className="underline underline-offset-4" href="https://brew.sh/" target="_blank" rel="noreferrer">
              Install Homebrew first
            </Link>
            .
          </Callout>
        </div>
      </Tab>
    </Tabs>
  );
}
