import { RootProvider } from 'fumadocs-ui/provider/next';
import { Analytics } from '@vercel/analytics/next';
import { GoogleAnalytics } from '@next/third-parties/google';
import './global.css';
import { Inter } from 'next/font/google';
import type { Metadata } from 'next';

const inter = Inter({
  subsets: ['latin'],
});

export const metadata: Metadata = {
  metadataBase: new URL('https://mato.sh'),
  title: {
    default: 'Mato - Multi-Agent Terminal Office',
    template: '%s | Mato',
  },
  description:
    'Mato is a terminal multiplexer and workspace for AI-agent workflows. Manage hundreds of sessions with live activity signals, persistence, and zero-conflict ergonomics.',
  applicationName: 'Mato',
  keywords: [
    'terminal multiplexer',
    'AI agents',
    'developer tools',
    'CLI workspace',
    'terminal productivity',
    'mato',
  ],
  alternates: {
    canonical: '/',
  },
  openGraph: {
    type: 'website',
    url: '/',
    siteName: 'Mato',
    title: 'Mato - Multi-Agent Terminal Office',
    description:
      'Manage hundreds of AI agent sessions from your terminal with live signals and daemon-backed persistence.',
    images: [
      {
        url: '/screenshot-coding.png',
        width: 2338,
        height: 1480,
        alt: 'Mato terminal workspace',
      },
    ],
  },
  twitter: {
    card: 'summary_large_image',
    title: 'Mato - Multi-Agent Terminal Office',
    description:
      'Manage hundreds of AI agent sessions from your terminal with live signals and daemon-backed persistence.',
    images: ['/screenshot-coding.png'],
  },
  robots: {
    index: true,
    follow: true,
  },
};

export default function Layout({ children }: LayoutProps<'/'>) {
  return (
    <html lang="en" className={inter.className} suppressHydrationWarning>
      <body className="flex flex-col min-h-screen">
        <RootProvider>{children}</RootProvider>
        <Analytics />
        <GoogleAnalytics gaId="G-Q30J2ZFNE4" />
      </body>
    </html>
  );
}
