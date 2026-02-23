'use client';

import Link from 'next/link';
import { useEffect, useState } from 'react';

export function VersionBadge() {
  const [version, setVersion] = useState('unknown');

  useEffect(() => {
    let mounted = true;
    fetch('/version.txt', { cache: 'no-store' })
      .then(async (res) => {
        if (!res.ok) return;
        const text = (await res.text()).trim();
        if (mounted && text) setVersion(text);
      })
      .catch(() => {});

    return () => {
      mounted = false;
    };
  }, []);

  const releaseHref =
    version === 'unknown'
      ? 'https://github.com/mr-kelly/mato/releases/latest'
      : `https://github.com/mr-kelly/mato/releases/tag/v${version}`;

  return (
    <Link
      href={releaseHref}
      target="_blank"
      rel="noreferrer"
      className="text-fd-muted-foreground/80 underline underline-offset-4 hover:text-fd-foreground"
      aria-label={`Open Mato release v${version}`}
    >
      Mato v{version}: Multi-Agent Terminal Office
    </Link>
  );
}
