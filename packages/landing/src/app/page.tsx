import type { Metadata, NextPage } from 'next';
import CTA from '@/components/molecules/CTA';
import Features, { type Feature } from '@/components/molecules/Features';
import Hero from '@/components/molecules/Hero';
import SectionHead from '@/components/molecules/SectionHead';
import Terminal from '@/components/molecules/Terminal';
import {
  LuCaptions,
  LuDownload,
  LuFileText,
  LuLock,
  LuZap,
} from 'react-icons/lu';

export const metadata: Metadata = {
  alternates: { canonical: '/' },
};

const GITHUB = 'https://github.com/hieudoanm/transcribers';

const FEATURES: Feature[] = [
  {
    icon: LuLock,
    title: 'Fully local',
    body: 'No uploads, no API keys, no accounts. Audio is extracted and transcribed on your own hardware.',
  },
  {
    icon: LuFileText,
    title: 'Four output formats',
    body: 'Plain text, SRT and WebVTT subtitles, or JSON with millisecond timestamps.',
  },
  {
    icon: LuDownload,
    title: 'Models on demand',
    body: 'Pick tiny through large-v3 by name; missing models download once into a local cache.',
  },
  {
    icon: LuZap,
    title: 'One small binary',
    body: 'whisper.cpp is compiled in — about 1.6 MB, statically linked, only ffmpeg required.',
  },
  {
    icon: LuCaptions,
    title: 'ffmpeg pipeline',
    body: 'Any container ffmpeg reads — mp4, mkv, mov, wav, mp3 — is decoded to 16 kHz mono WAV before transcription.',
  },
  {
    icon: LuCaptions,
    title: 'Subtitle-ready',
    body: 'SRT and WebVTT output with millisecond timestamps, ready for players and video editors.',
  },
];

const HERO_LINES = [
  '$ transcribers talk.mp4 -f srt -o talk',
  'wrote talk.srt',
  '',
  '1',
  '00:00:00,000 --> 00:00:05,200',
  'Delete filler badges, drop redundant labels…',
];

const DEMO_TERMINALS: {
  caption: string;
  lines: string[];
  wide?: boolean;
}[] = [
  {
    caption: 'Transcribe to SRT',
    wide: true,
    lines: [
      '$ transcribers talk.mp4 -f srt -o talk',
      'wrote talk.srt',
      '',
      '1',
      '00:00:00,000 --> 00:00:05,200',
      'Delete filler badges, drop redundant labels, un-etalize the',
      'random word in your header.',
      '',
      '2',
      '00:00:05,200 --> 00:00:12,640',
      'Only use mono where necessary. Use a design system, show the',
      'actual product, and large the main feature.',
    ],
  },
  {
    caption: 'SRT to stdout (default)',
    lines: [
      '$ transcribers talk.mp4',
      '1',
      '00:00:00,000 --> 00:00:05,200',
      'Delete filler badges, drop redundant labels…',
    ],
  },
  {
    caption: 'Pick a model',
    lines: [
      '$ transcribers talk.mp4 --model small',
      'downloading model ggml-small.bin ...',
      'transcribing... 100%',
    ],
  },
];

const INSTALL_LINES = [
  '$ curl -fsSL https://raw.githubusercontent.com/hieudoanm/transcribers/main/scripts/install.sh | bash',
  'Installed to /usr/local/bin/transcribers',
  '$ transcribers talk.mp4 -f srt -o talk',
  'wrote talk.srt',
];

const HomePage: NextPage = () => (
  <main>
    <Hero
      title="Turn video and audio into text, offline."
      description="One Rust binary extracts the audio track with ffmpeg and transcribes it with whisper.cpp — mp4, mkv, mov, wav, mp3 and more, with no cloud and no account."
      primary={{ label: 'Get started', href: '#install' }}
      secondary={{ label: 'See it in action', href: '#demo' }}
      stats={[
        { value: '4', label: 'output formats' },
        { value: '~1.6 MB', label: 'single binary' },
        { value: '0', label: 'cloud calls' },
      ]}>
      <Terminal lines={HERO_LINES} />
    </Hero>

    <Features
      title="Everything happens on your machine."
      description="Your media never leaves your disk. The model runs locally through whisper.cpp."
      features={FEATURES}
    />

    <section id="demo" className="bg-base-200 py-20">
      <div className="mx-auto w-full max-w-6xl px-5">
        <SectionHead
          title="See it in action"
          description="Real CLI output — SRT subtitles straight to stdout or a file."
        />
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
          {DEMO_TERMINALS.map((t) => (
            <Terminal
              key={t.caption}
              lines={t.lines}
              className={t.wide ? 'lg:col-span-2' : ''}
            />
          ))}
        </div>
      </div>
    </section>

    <section id="install" className="bg-base-200 py-20">
      <div className="mx-auto w-full max-w-3xl px-5">
        <SectionHead
          title="Install in one command"
          description="Prebuilt binaries for macOS and Linux, or build from source with Cargo. Requires ffmpeg on PATH."
        />
        <Terminal lines={INSTALL_LINES} />
        <p className="text-base-content/60 mt-6 text-center text-sm">
          Or from source:{' '}
          <code className="bg-base-300 rounded px-1.5 py-0.5 font-mono text-xs">
            cargo install --git https://github.com/hieudoanm/transcribers
            transcribers
          </code>{' '}
          (needs cmake for the whisper.cpp build). See the{' '}
          <a
            href={`${GITHUB}#readme`}
            target="_blank"
            rel="noreferrer"
            className="text-primary font-semibold">
            README
          </a>{' '}
          for details.
        </p>
      </div>
    </section>

    <CTA
      title="Ready to transcribe?"
      description="One binary, fully local — video and audio to text."
      primary={{ label: 'View on GitHub', href: GITHUB, external: true }}
      secondary={{
        label: 'Read the README',
        href: `${GITHUB}#readme`,
        external: true,
      }}
    />
  </main>
);

export default HomePage;
