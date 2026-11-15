import type { Metadata, NextPage } from 'next';
import CTA from '@/components/molecules/CTA';
import Features, { type Feature } from '@/components/molecules/Features';
import Hero from '@/components/molecules/Hero';
import SectionHead from '@/components/molecules/SectionHead';
import Terminal from '@/components/molecules/Terminal';
import {
  LuBot,
  LuCaptions,
  LuDownload,
  LuFileText,
  LuGlobe,
  LuLock,
  LuMusic,
  LuServer,
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
  {
    icon: LuGlobe,
    title: 'URLs in, transcripts out',
    body: 'Pass a YouTube video, Short or Instagram reel to --url — yt-dlp fetches it, then the same local pipeline runs.',
  },
  {
    icon: LuMusic,
    title: 'Audio extraction',
    body: '--audio writes wav, mp3 or m4a next to the transcript; --audio-only converts without transcribing.',
  },
  {
    icon: LuBot,
    title: 'MCP server',
    body: 'transcribers mcp lets agents transcribe media and list models through the same local pipeline — no extra setup, no uploads.',
  },
  {
    icon: LuServer,
    title: 'HTTP server',
    body: 'transcribers serve exposes POST /transcribe — upload a file or send a URL, get the transcript back as JSON or subtitles.',
  },
];

const HERO_LINES = [
  '$ transcribers --file talk.mp4 -f srt -o talk',
  'wrote talk.srt',
  '',
  '1',
  '00:00:00,000 --> 00:00:05,200',
  'Delete filler badges, drop redundant labels…',
];

const DEMO_TERMINALS: {
  caption: string;
  lines: string[];
}[] = [
  {
    caption: 'Transcribe to SRT',
    lines: [
      '$ transcribers --file talk.mp4 -f srt -o talk',
      'wrote talk.srt',
      '',
      '1',
      '00:00:00,000 --> 00:00:05,200',
      'Delete filler badges, drop redundant labels…',
    ],
  },
  {
    caption: 'From a URL',
    lines: [
      '$ transcribers --url "https://youtu.be/jNQXAC9IVRw"',
      'downloading https://youtu.be/jNQXAC9IVRw ...',
      'transcribing... 100%',
      'wrote yt-jNQXAC9IVRw.srt',
      ' ',
      ' ',
    ],
  },
  {
    caption: 'Audio extraction',
    lines: [
      '$ transcribers --file talk.mp4 -f srt -o talk --audio m4a',
      'wrote talk.srt',
      'wrote talk.m4a',
      ' ',
      ' ',
      ' ',
    ],
  },
  {
    caption: 'SRT to stdout (default)',
    lines: [
      '$ transcribers --file talk.mp4',
      '1',
      '00:00:00,000 --> 00:00:05,200',
      'Delete filler badges, drop redundant labels…',
      ' ',
      ' ',
    ],
  },
  {
    caption: 'Pick a model',
    lines: [
      '$ transcribers --file talk.mp4 --model small',
      'downloading model ggml-small.bin ...',
      'transcribing... 100%',
      'wrote talk.srt',
      ' ',
      ' ',
    ],
  },
  {
    caption: 'HTTP server',
    lines: [
      '$ transcribers serve --port 8080',
      'serving on http://127.0.0.1:8080',
      '$ curl -F "file=@talk.mp4" http://127.0.0.1:8080/transcribe',
      '1',
      '00:00:00,000 --> 00:00:05,200',
      'Delete filler badges, drop redundant labels…',
    ],
  },
];


const INSTALL_LINES = [
  '$ curl -fsSL https://raw.githubusercontent.com/hieudoanm/transcribers/master/scripts/install.sh | bash',
  'Installed to /usr/local/bin/transcribers',
  '$ transcribers --file talk.mp4 -f srt -o talk',
  'wrote talk.srt',
];

const HomePage: NextPage = () => (
  <main>
    <Hero
      title="Turn video and audio into text, offline."
      description="One Rust binary extracts the audio track with ffmpeg and transcribes it with whisper.cpp — local files or YouTube/Instagram URLs, with no cloud and no account."
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
          description="Real CLI output — files, URLs, audio, models and the HTTP server."
        />
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
          {DEMO_TERMINALS.map((t) => (
            <Terminal key={t.caption} lines={t.lines} />
          ))}
        </div>
      </div>
    </section>

    <section id="install" className="py-20">
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
