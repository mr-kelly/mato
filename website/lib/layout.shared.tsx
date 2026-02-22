import type { BaseLayoutProps } from 'fumadocs-ui/layouts/shared';
import Link from 'next/link';

export const gitConfig = {
  user: 'mr-kelly',
  repo: 'mato',
  branch: 'main',
};

export function baseOptions(): BaseLayoutProps {
  return {
    nav: {
      title: (
        <Link href="/" className="mato-nav-brand">
          <img src="/logo.svg" alt="Mato logo" width={24} height={24} />
          <span>Mato</span>
        </Link>
      ),
    },
    githubUrl: `https://github.com/${gitConfig.user}/${gitConfig.repo}`,
  };
}
