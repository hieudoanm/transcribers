import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import type { FC, ReactNode } from 'react';
import Footer from '@/components/molecules/Footer';
import Navbar from '@/components/molecules/Navbar';
import './globals.css';

const inter = Inter({ subsets: ['latin'] });

const GITHUB = 'https://github.com/hieudoanm/transcribers';
const BASE = process.env.NODE_ENV === 'production' ? '/transcribers' : '';
const LOGO =
  process.env.NODE_ENV === 'production'
    ? '/transcribers/favicon.svg'
    : '/favicon.svg';

const NAV_LINKS = [
  { label: 'Features', href: '#features' },
  { label: 'Demo', href: '#demo' },
  { label: 'Install', href: '#install' },
  { label: 'Docs', href: '/docs/' },
  { label: 'Downloads', href: '/downloads/' },
];

const DESCRIPTION =
  'Extract transcripts from video and audio files with a single Rust binary. ffmpeg + whisper.cpp, fully local.';

export const metadata: Metadata = {
  metadataBase: new URL('https://hieudoanm.github.io/transcribers/'),
  title: {
    default: 'transcribers — video and audio to text, locally',
    template: '%s — transcribers',
  },
  description: DESCRIPTION,
  keywords: [
    'transcription',
    'speech to text',
    'Whisper',
    'audio to text',
    'video to text',
    'local transcription',
    'Rust',
    'MCP server',
  ],
  authors: [
    { name: 'hieudoan', url: 'https://github.com/hieudoanm/transcribers' },
  ],
  category: 'software',
  openGraph: {
    siteName: 'transcribers',
    type: 'website',
    title: 'transcribers — video and audio to text, locally',
    description: DESCRIPTION,
    url: 'https://hieudoanm.github.io/transcribers/',
    images: ['/og.png'],
  },
  twitter: { card: 'summary_large_image', images: ['/og.png'] },
  icons: {
    icon: [
      { url: `${BASE}/favicon.ico`, sizes: 'any' },
      { url: `${BASE}/favicon.svg`, type: 'image/svg+xml' },
    ],
  },
};

const JSON_LD = {
  '@context': 'https://schema.org',
  '@graph': [
    {
      '@type': 'WebSite',
      name: 'transcribers',
      url: 'https://hieudoanm.github.io/transcribers/',
      description:
        'Extract transcripts from video and audio files with a single Rust binary. ffmpeg + whisper.cpp, fully local.',
    },
    {
      '@type': 'SoftwareApplication',
      name: 'transcribers',
      applicationCategory: 'MultimediaApplication',
      description:
        'Extract transcripts from video and audio files with a single Rust binary. ffmpeg + whisper.cpp, fully local.',
      url: 'https://hieudoanm.github.io/transcribers/',
      codeRepository: 'https://github.com/hieudoanm/transcribers',
      offers: { '@type': 'Offer', price: '0', priceCurrency: 'USD' },
    },
  ],
};

interface RootLayoutProps {
  children: ReactNode;
}

const RootLayout: FC<RootLayoutProps> = ({ children }) => (
  <html
    lang="en"
    data-theme="transcribers"
    className={`scroll-smooth antialiased motion-reduce:scroll-auto ${inter.className}`}>
    <body className="bg-base-100 text-base-content m-0 leading-relaxed">
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(JSON_LD) }}
      />
      <Navbar
        name="transcribers"
        logoSrc={LOGO}
        links={NAV_LINKS}
        githubUrl={GITHUB}
      />
      {children}
      <Footer
        name="transcribers"
        tagline="ffmpeg + whisper.cpp, fully local"
        githubUrl={GITHUB}
      />
    </body>
  </html>
);

export default RootLayout;
