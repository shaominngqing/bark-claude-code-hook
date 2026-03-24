// Bark Demo Video — Theme & Constants
// All colors, gradients, and shared style tokens

export const COLORS = {
  // Primary gradient: matches install.sh _gradient() colors 39-44
  gradientStart: '#38bdf8',   // sky-400 (visual approximation of ANSI 39 #00afff)
  gradientMid: '#818cf8',     // indigo-400
  gradientEnd: '#c084fc',     // purple-400

  // Script exact ANSI 256 gradient colors (39-44) used by _gradient()
  ansi39: '#00afff',
  ansi40: '#00d7d7',
  ansi41: '#00d7ff',
  ansi42: '#00ffaf',
  ansi43: '#00ffd7',
  ansi44: '#00ffff',

  // Script named colors: C1/C2/C3/C4
  c1: '#00afff',    // 38;5;39
  c2: '#00d7ff',    // 38;5;45
  c3: '#00ffff',    // 38;5;51
  c4: '#5fffff',    // 38;5;87

  // Terminal
  termBg: '#0d1117',
  termBgLight: '#161b22',
  termBorder: '#30363d',
  termText: '#e6edf3',
  termDim: '#7d8590',
  termPrompt: '#58a6ff',

  // Risk levels (match script: GREEN/YELLOW/RED)
  low: '#3fb950',          // ANSI green
  lowBg: 'rgba(63, 185, 80, 0.15)',
  medium: '#d29922',       // ANSI yellow (bold)
  mediumBg: 'rgba(210, 153, 34, 0.15)',
  high: '#f85149',         // ANSI red
  highBg: 'rgba(248, 81, 73, 0.15)',

  // Script extra colors
  accent: '#ff87ff',       // ANSI 213 — ACCENT in script
  orange: '#ff8700',       // ANSI 208 — ORANGE in script (AI source)
  purple: '#af87ff',       // ANSI 141 — PURPLE in script (RULE source)

  white: '#ffffff',
  black: '#000000',

  // macOS notification
  notifBg: '#2d2d2d',
  notifBorder: '#444',
} as const;

// The 6-step gradient cycle used by _gradient() / _gradient_line() in install.sh
export const GRADIENT_CYCLE = [
  COLORS.ansi39, COLORS.ansi40, COLORS.ansi41,
  COLORS.ansi42, COLORS.ansi43, COLORS.ansi44,
];

export const GRADIENT_CSS = `linear-gradient(135deg, ${COLORS.gradientStart}, ${COLORS.gradientMid}, ${COLORS.gradientEnd})`;

// Video settings
export const FPS = 30;
export const WIDTH = 1920;
export const HEIGHT = 1080;

// Scene durations in seconds
export const SCENE_DURATIONS = {
  opening: 3,
  install: 5,
  workflow: 30,      // read-only → edit → AI assess → cache hit → high risk
  stats: 5,
  cacheManage: 3,
  logView: 3,
  onOff: 4,
  help: 3,
} as const;

export const TOTAL_DURATION = Object.values(SCENE_DURATIONS).reduce((a, b) => a + b, 0);

// Figlet slant ASCII art for Bark
export const BARK_ASCII = [
  '    ____             __',
  '   / __ )____ ______/ /__',
  '  / __  / __ `/ ___/ //_/',
  ' / /_/ / /_/ / /  / ,<',
  '/_____/\\__,_/_/  /_/|_|',
];

// Small version ASCII art
export const BARK_ASCII_SMALL = [
  ' ___           _',
  '| _ ) __ _ _ _| |__',
  '| _ \\/ _` | \'_| / /',
  '|___/\\__,_|_| |_\\_\\',
];

// Font
export const FONT_MONO = "'SF Mono', 'Cascadia Code', 'Fira Code', 'JetBrains Mono', Menlo, Consolas, monospace";
export const FONT_SANS = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif";

// Shadow & glow tokens
export const SHADOWS = {
  termWindow: '0 25px 80px rgba(0,0,0,0.55), 0 10px 30px rgba(0,0,0,0.3)',
  termWindowHover: '0 30px 90px rgba(0,0,0,0.6), 0 0 40px rgba(56,189,248,0.08)',
  notification: '0 12px 40px rgba(0,0,0,0.65), 0 0 0 1px rgba(255,255,255,0.06)',
  glowLow: `0 0 20px rgba(63, 185, 80, 0.4), 0 0 60px rgba(63, 185, 80, 0.1)`,
  glowMedium: `0 0 20px rgba(210, 153, 34, 0.4), 0 0 60px rgba(210, 153, 34, 0.1)`,
  glowHigh: `0 0 20px rgba(248, 81, 73, 0.5), 0 0 80px rgba(248, 81, 73, 0.15)`,
  glowBlue: `0 0 30px rgba(56, 189, 248, 0.3), 0 0 80px rgba(56, 189, 248, 0.1)`,
} as const;
