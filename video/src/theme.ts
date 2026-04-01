// Bark Demo Video — Theme & Constants
// All colors, gradients, and shared style tokens

export const COLORS = {
  // Primary gradient: matches install.sh _gradient() colors 39-44
  gradientStart: '#38bdf8',   // sky-400
  gradientMid: '#818cf8',     // indigo-400
  gradientEnd: '#c084fc',     // purple-400

  // Script exact ANSI 256 gradient colors (39-44)
  ansi39: '#00afff',
  ansi40: '#00d7d7',
  ansi41: '#00d7ff',
  ansi42: '#00ffaf',
  ansi43: '#00ffd7',
  ansi44: '#00ffff',

  // Script named colors
  c1: '#00afff',
  c2: '#00d7ff',
  c3: '#00ffff',
  c4: '#5fffff',

  // Terminal
  termBg: '#0d1117',
  termBgLight: '#161b22',
  termBorder: '#30363d',
  termText: '#e6edf3',
  termDim: '#7d8590',
  termPrompt: '#58a6ff',

  // Risk levels
  low: '#3fb950',
  lowBg: 'rgba(63, 185, 80, 0.15)',
  medium: '#d29922',
  mediumBg: 'rgba(210, 153, 34, 0.15)',
  high: '#f85149',
  highBg: 'rgba(248, 81, 73, 0.15)',

  // Extra
  accent: '#ff87ff',
  orange: '#ff8700',
  purple: '#af87ff',
  white: '#ffffff',
  black: '#000000',
  notifBg: '#2d2d2d',
  notifBorder: '#444',
} as const;

export const GRADIENT_CYCLE = [
  COLORS.ansi39, COLORS.ansi40, COLORS.ansi41,
  COLORS.ansi42, COLORS.ansi43, COLORS.ansi44,
];

export const GRADIENT_CSS = `linear-gradient(135deg, ${COLORS.gradientStart}, ${COLORS.gradientMid}, ${COLORS.gradientEnd})`;

// Video settings
export const FPS = 60;
export const WIDTH = 1920;
export const HEIGHT = 1080;

// Scene durations in seconds
export const SCENE_DURATIONS = {
  intro: 6,        // logo + pain point
  recording: 85,   // full demo recording (no cuts)
  closing: 8,      // CTA + GitHub
} as const;

export const TOTAL_DURATION = Object.values(SCENE_DURATIONS).reduce((a, b) => a + b, 0); // 100s

// ASCII art (kept for potential use)
export const BARK_ASCII = `\
 ███████████                      █████
░░███░░░░░███                    ░░███
 ░███    ░███  ██████   ████████  ░███ █████
 ░██████████  ░░░░░███ ░░███░░███ ░███░░███
 ░███░░░░░███  ███████  ░███ ░░░  ░██████░
 ░███    ░███ ███░░███  ░███      ░███░░███
 ███████████ ░░████████ █████     ████ █████
░░░░░░░░░░░   ░░░░░░░░ ░░░░░     ░░░░ ░░░░░`;

// Typography
export const FONT = {
  mono: "'SF Mono', 'Cascadia Code', 'Fira Code', 'JetBrains Mono', Menlo, Consolas, monospace",
  sans: "-apple-system, 'SF Pro Display', 'Helvetica Neue', Arial, sans-serif",
} as const;

// Shadows
export const SHADOW = {
  terminal: '0 25px 80px rgba(0,0,0,0.55)',
  glow: (color: string) => `0 0 40px ${color}40, 0 0 80px ${color}20`,
} as const;
