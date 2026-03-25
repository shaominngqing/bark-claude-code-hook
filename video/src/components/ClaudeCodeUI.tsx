import React, { CSSProperties } from 'react';
import { useCurrentFrame, useVideoConfig, spring, interpolate } from 'remotion';
import { COLORS } from '../theme';

/**
 * Pixel-perfect Claude Code UI based on official website screenshot.
 *
 * Key details from screenshot:
 * - Background: dark gray #1e1e1e (NOT pure black)
 * - Title bar: darker gray #2a2a2a, dots are GRAY (unfocused window), no title text
 * - Logo: red pixel art ~48px, larger than before
 * - Header: "Claude Code" WHITE (not green!) + "v2.1.76" gray
 *           "Opus 4.6 (1M Context) · Claude Enterprise" gray
 *           "/Users/johnnie/taskflow" gray
 * - User input: ">" prompt, entire line has gray background highlight
 * - Claude response: "●" white bullet + white text
 * - Tool call: "●" GREEN bullet + GREEN bold "ToolName(args)"
 * - Tool result: "L Done (17 tool uses · 38.0k tokens · 28s)" — capital L not └
 * - Thinking: "* Clauding... (esc to interrupt)" — star is orange
 * - Font: Menlo, ~14-15px, line-height ~1.8
 * - Generous spacing between elements
 */

const MONO = "'Menlo', 'SF Mono', 'Monaco', monospace";
const BG = '#1e1e1e';
const TITLE_BAR_BG = '#2a2a2a';
const TEXT = '#e0e0e0';
const DIM = '#808080';
const TOOL_GREEN = '#4ec970';
const INPUT_BG = '#2a2d30';

// ─────────── Terminal Window ───────────
interface ClaudeTerminalProps {
  children: React.ReactNode;
  width?: number;
  height?: number;
  style?: CSSProperties;
  enterDelay?: number;
  title?: string;
}

export const ClaudeTerminal: React.FC<ClaudeTerminalProps> = ({
  children,
  width = 1300,
  height = 780,
  style,
  enterDelay = 0,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const entered = enterDelay > 0 ? spring({
    frame: Math.max(0, frame - enterDelay),
    fps,
    config: { damping: 22, mass: 1, stiffness: 160 },
  }) : 1;

  const opacity = interpolate(entered, [0, 1], [0, 1]);
  const scl = interpolate(entered, [0, 1], [0.95, 1]);
  const ty = interpolate(entered, [0, 1], [20, 0]);

  return (
    <div style={{
      width, height, borderRadius: 10, overflow: 'hidden',
      display: 'flex', flexDirection: 'column',
      opacity, transform: `translateY(${ty}px) scale(${scl})`,
      boxShadow: '0 22px 70px 4px rgba(0,0,0,0.56), 0 0 0 0.5px rgba(0,0,0,0.3)',
      ...style,
    }}>
      {/* Title bar — gray dots (unfocused), no text */}
      <div style={{
        height: 36, background: TITLE_BAR_BG,
        display: 'flex', alignItems: 'center', paddingLeft: 14,
        flexShrink: 0, borderBottom: `1px solid #1a1a1a`,
      }}>
        {[
          { bg: '#ff5f57', border: '#e0443e' },
          { bg: '#febc2e', border: '#dea123' },
          { bg: '#28c840', border: '#1aab29' },
        ].map((d, i) => (
          <div key={i} style={{
            width: 12, height: 12, borderRadius: '50%',
            background: d.bg, marginRight: 8, border: `0.5px solid ${d.border}`,
          }} />
        ))}
      </div>

      {/* Body */}
      <div style={{
        flex: 1, background: BG, padding: '16px 24px',
        fontFamily: MONO, fontSize: 15, lineHeight: 1.85, color: TEXT,
        overflow: 'hidden', position: 'relative',
      }}>
        {children}
      </div>
    </div>
  );
};

// ─────────── Startup Welcome Card ───────────
// CSS border card layout, only logo uses Unicode text
export const ClaudeCodeHeader: React.FC<{ delay?: number }> = ({ delay = 0 }) => {
  const frame = useCurrentFrame();
  if (frame < delay) return null;

  const BCOLOR = '#444';
  const G = '#3fb950';
  const R = '#d4634a';

  return (
    <div style={{
      border: `1px solid ${BCOLOR}`, borderRadius: 8,
      fontFamily: MONO, fontSize: 13, marginBottom: 14,
      overflow: 'hidden',
    }}>
      {/* Title bar */}
      <div style={{
        borderBottom: `1px solid ${BCOLOR}`, padding: '6px 14px',
        color: TEXT, fontSize: 13,
      }}>
        Claude Code v2.1.81
      </div>

      {/* Two-column body */}
      <div style={{ display: 'flex', minHeight: 140 }}>
        {/* Left: welcome + logo + info */}
        <div style={{
          flex: '0 0 48%', borderRight: `1px solid ${BCOLOR}`,
          display: 'flex', flexDirection: 'column', alignItems: 'center',
          justifyContent: 'center', padding: '12px 16px', gap: 6,
        }}>
          <div style={{ color: G, fontWeight: 700, fontSize: 15 }}>Welcome back!</div>
          {/* Unicode pixel logo */}
          <div style={{
            color: R, fontFamily: MONO, fontSize: 22,
            lineHeight: 1, margin: '8px 0', whiteSpace: 'pre',
          }}>{' ▐▛███▜▌\n▝▜█████▛▘\n  ▘▘ ▝▝'}</div>
          <div style={{ color: DIM, fontSize: 11, textAlign: 'center' }}>
            claude-opus-4-6 · API Usage Billing
          </div>
          <div style={{ color: DIM, fontSize: 11 }}>~/project</div>
        </div>

        {/* Right: tips + recent */}
        <div style={{ flex: 1, padding: '12px 16px' }}>
          <div style={{ color: G, fontSize: 13, marginBottom: 4 }}>Tips for getting started</div>
          <div style={{ color: TEXT, fontSize: 12, marginBottom: 12, lineHeight: 1.5 }}>
            Run /init to create a CLAUDE.md file with instructions for Claude
          </div>
          <div style={{ height: 1, background: BCOLOR, marginBottom: 12 }} />
          <div style={{ color: G, fontSize: 13, marginBottom: 4 }}>Recent activity</div>
          <div style={{ color: DIM, fontSize: 12 }}>No recent activity</div>
        </div>
      </div>
    </div>
  );
};

// ─────────── Input Box (inline) ───────────
interface InputBoxProps {
  text: string;
  delay?: number;
  typingSpeed?: number;
  submitFrame?: number;
}

export const InputBox: React.FC<InputBoxProps> = ({
  text, delay = 0, typingSpeed = 1.5, submitFrame,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  if (frame < delay) return null;

  const submitted = submitFrame != null && frame >= submitFrame;

  let chars = 0;
  if (!submitted) {
    const elapsed = frame - delay;
    let acc = 0;
    for (let i = 0; i < text.length; i++) {
      const ch = text[i];
      const mult = ch === ' ' ? 0.55 : ch === '"' ? 0.4 : 1.0;
      acc += 1 / (typingSpeed * mult);
      if (acc > elapsed) break;
      chars++;
    }
    chars = Math.min(chars, text.length);
  }

  const cursorOn = Math.floor(frame / Math.round(fps * 0.53)) % 2 === 0;

  return (
    <div style={{ marginTop: 8 }}>
      {/* Top separator line */}
      <div style={{ height: 1, background: '#333', marginBottom: 8 }} />
      {/* Input line: > text | */}
      <div style={{ display: 'flex', alignItems: 'center', minHeight: 22, paddingLeft: 2 }}>
        <span style={{ color: DIM, marginRight: 8, fontWeight: 600 }}>&gt;</span>
        {!submitted && (
          <span style={{ color: TEXT }}>{text.slice(0, chars)}</span>
        )}
        <span style={{
          display: 'inline-block', width: 2, height: 17,
          background: TEXT, marginLeft: 1,
          opacity: cursorOn ? 1 : 0,
        }} />
      </div>
      {/* Bottom separator line */}
      <div style={{ height: 1, background: '#333', marginTop: 8 }} />
    </div>
  );
};

// ─────────── Submitted User Message ───────────
// After enter: shows as "> text" with gray background
export const UserMessage: React.FC<{
  text: string;
  delay?: number;
}> = ({ text, delay = 0 }) => {
  const frame = useCurrentFrame();
  if (frame < delay) return null;

  return (
    <ClaudeActivity delay={delay} style={{ marginBottom: 4 }}>
      <span style={{ color: DIM, fontWeight: 600 }}>&gt; </span>
      <span style={{ color: TEXT }}>{text}</span>
    </ClaudeActivity>
  );
};

// ─────────── Claude Response "● text" ───────────
export const ClaudeResponse: React.FC<{
  text: string;
  delay?: number;
  wordsPerFrame?: number;
  style?: CSSProperties;
}> = ({ text, delay = 0, wordsPerFrame = 0.4, style }) => {
  const frame = useCurrentFrame();
  if (frame < delay) return null;

  const elapsed = frame - delay;
  const words = text.split(' ');
  const wordsToShow = Math.min(Math.floor(elapsed * wordsPerFrame), words.length);

  return (
    <ClaudeActivity delay={delay} style={{ marginTop: 8, ...style }}>
      <span style={{ color: TEXT }}>● </span>
      <span style={{ color: TEXT }}>{words.slice(0, wordsToShow).join(' ')}</span>
    </ClaudeActivity>
  );
};

// ─────────── Tool Call "● ToolName(args)" ───────────
// Green dot + green bold name — matches screenshot exactly
export const ToolCall: React.FC<{
  tool: string;
  args: string;
  delay?: number;
  style?: CSSProperties;
}> = ({ tool, args, delay = 0, style }) => {
  const frame = useCurrentFrame();
  if (frame < delay) return null;

  return (
    <ClaudeActivity delay={delay} style={{ marginTop: 6, ...style }}>
      <span style={{ color: TOOL_GREEN }}>● </span>
      <span style={{ color: TOOL_GREEN, fontWeight: 700 }}>{tool}</span>
      <span style={{ color: DIM }}>({args})</span>
    </ClaudeActivity>
  );
};

// ─────────── Tool Result "L Done (...)" ───────────
// Capital L (not └), indented — matches screenshot
export const ToolResult: React.FC<{
  text: string;
  delay?: number;
  color?: string;
  style?: CSSProperties;
}> = ({ text, delay = 0, color = DIM, style }) => {
  const frame = useCurrentFrame();
  if (frame < delay) return null;

  return (
    <ClaudeActivity delay={delay} style={{ paddingLeft: 18, ...style }}>
      <span style={{ color: DIM }}>L </span>
      <span style={{ color }}>{text}</span>
    </ClaudeActivity>
  );
};

// ─────────── Thinking "* Clauding... (esc to interrupt)" ───────────
export const ClaudeThinking: React.FC<{
  delay?: number;
  endFrame?: number;
}> = ({ delay = 0, endFrame }) => {
  const frame = useCurrentFrame();
  if (frame < delay) return null;
  if (endFrame && frame >= endFrame) return null;

  // Dots animation
  const dots = '.'.repeat((Math.floor((frame - delay) / 8) % 3) + 1);

  return (
    <ClaudeActivity delay={delay} style={{ marginTop: 6 }}>
      <span style={{ color: '#c0884a' }}>* </span>
      <span style={{ color: '#c0884a' }}>Clauding{dots}</span>
      <span style={{ color: DIM }}> (esc to interrupt)</span>
    </ClaudeActivity>
  );
};

// ─────────── Bark Assessment Result ───────────
export const BarkResult: React.FC<{
  level: 'low' | 'medium' | 'high';
  text: string;
  delay?: number;
  cached?: boolean;
}> = ({ level, text, delay = 0, cached = false }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  if (frame < delay) return null;

  const entered = spring({
    frame: frame - delay, fps,
    config: { damping: 18, mass: 0.6, stiffness: 320 },
  });
  const opacity = interpolate(entered, [0, 1], [0, 1]);
  const translateX = interpolate(entered, [0, 1], [15, 0]);

  const cfg = {
    low: { fg: COLORS.low, icon: '✓', label: 'Low' },
    medium: { fg: COLORS.medium, icon: '⚠', label: 'Medium' },
    high: { fg: COLORS.high, icon: '🚨', label: 'High' },
  }[level];

  return (
    <span style={{
      opacity, transform: `translateX(${translateX}px)`,
      display: 'inline-flex', alignItems: 'center', gap: 4,
      fontSize: 13, marginLeft: 12,
    }}>
      <span style={{ color: cfg.fg }}>{cfg.icon} [{cfg.label}]</span>
      <span style={{ color: DIM }}>{text}</span>
      {cached && <span style={{ color: '#555' }}>(cached)</span>}
    </span>
  );
};

// ─────────── Generic Animated Line ───────────
export const ClaudeActivity: React.FC<{
  children: React.ReactNode;
  delay?: number;
  style?: CSSProperties;
}> = ({ children, delay = 0, style }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  if (frame < delay) return null;

  const entered = spring({
    frame: frame - delay, fps,
    config: { damping: 30, mass: 0.8, stiffness: 300 },
  });
  const opacity = interpolate(entered, [0, 1], [0, 1]);
  const translateY = interpolate(entered, [0, 1], [6, 0]);

  return (
    <div style={{ opacity, transform: `translateY(${translateY}px)`, ...style }}>
      {children}
    </div>
  );
};

// ─────────── Spinner (for Bark AI assessment) ───────────
export const Spinner: React.FC<{
  text?: string;
  startFrame?: number;
}> = ({ text = 'AI assessing...', startFrame = 0 }) => {
  const frame = useCurrentFrame();
  const elapsed = frame - startFrame;
  if (elapsed < 0) return null;

  const FRAMES = ['▸▹▹▹▹', '▹▸▹▹▹', '▹▹▸▹▹', '▹▹▹▸▹', '▹▹▹▹▸'];
  const idx = Math.floor(elapsed / 2.4) % FRAMES.length;

  return (
    <span style={{ fontFamily: MONO, fontSize: 14 }}>
      <span style={{ color: TOOL_GREEN, letterSpacing: 1 }}>{FRAMES[idx]}</span>
      <span style={{ color: DIM, marginLeft: 8 }}>{text}</span>
    </span>
  );
};

// ─────────── Shell Prompt (for bark CLI scenes) ───────────
export const ShellPrompt: React.FC<{
  command: string;
  delay?: number;
  typingSpeed?: number;
}> = ({ command, delay = 0, typingSpeed = 2.0 }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  if (frame < delay) return null;

  const elapsed = frame - delay;
  let chars = 0;
  let acc = 0;
  for (let i = 0; i < command.length; i++) {
    const ch = command[i];
    const mult = ch === ' ' ? 0.55 : ch === '-' ? 0.5 : 1.0;
    acc += 1 / (typingSpeed * mult);
    if (acc > elapsed) break;
    chars++;
  }
  chars = Math.min(chars, command.length);
  const done = chars >= command.length;
  const cursorOn = Math.floor(frame / Math.round(fps * 0.53)) % 2 === 0;

  return (
    <div style={{ marginBottom: 6 }}>
      <span style={{ color: TOOL_GREEN }}>❯❯</span>
      <span style={{ color: '#e5c07b' }}> ~ </span>
      <span style={{ color: TEXT }}>{command.slice(0, chars)}</span>
      {(!done || cursorOn) && (
        <span style={{
          display: 'inline-block', width: 2, height: 17,
          background: TEXT, marginLeft: 1,
          opacity: !done ? 1 : cursorOn ? 0.7 : 0,
        }} />
      )}
    </div>
  );
};
