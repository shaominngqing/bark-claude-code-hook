import React, { CSSProperties } from 'react';
import { useCurrentFrame, useVideoConfig, spring, interpolate } from 'remotion';
import { COLORS, FONT_MONO } from '../theme';

const MONO = "'Menlo', 'SF Mono', 'Monaco', monospace";

// ─────────── Terminal Window Shell ───────────
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
  title = 'Fix login bug and push changes',
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const entered = enterDelay > 0 ? spring({
    frame: Math.max(0, frame - enterDelay),
    fps,
    config: { damping: 22, mass: 1, stiffness: 160 },
  }) : 1;

  const opacity = interpolate(entered, [0, 1], [0, 1]);
  const scale = interpolate(entered, [0, 1], [0.95, 1]);
  const translateY = interpolate(entered, [0, 1], [20, 0]);

  return (
    <div
      style={{
        width, height, borderRadius: 10, overflow: 'hidden',
        display: 'flex', flexDirection: 'column',
        opacity, transform: `translateY(${translateY}px) scale(${scale})`,
        boxShadow: '0 22px 70px 4px rgba(0,0,0,0.56), 0 0 0 0.5px rgba(0,0,0,0.3)',
        ...style,
      }}
    >
      {/* Title bar */}
      <div style={{
        height: 32, background: '#303030',
        display: 'flex', alignItems: 'center', paddingLeft: 12, paddingRight: 12,
        flexShrink: 0, position: 'relative', borderBottom: '1px solid #1a1a1a',
      }}>
        <div style={{ display: 'flex', gap: 8, zIndex: 1 }}>
          {[
            { bg: '#ff5f57', border: '#e0443e' },
            { bg: '#febc2e', border: '#dea123' },
            { bg: '#28c840', border: '#1aab29' },
          ].map((d, i) => (
            <div key={i} style={{
              width: 12, height: 12, borderRadius: '50%',
              background: d.bg, border: `0.5px solid ${d.border}`,
            }} />
          ))}
        </div>
        <div style={{
          position: 'absolute', left: 0, right: 0, textAlign: 'center',
          color: '#aaa', fontSize: 13, fontFamily: MONO, fontWeight: 500,
        }}>
          <span style={{ color: '#999' }}>✱</span> {title}
        </div>
      </div>

      {/* Terminal body */}
      <div style={{
        flex: 1, background: '#000000', padding: '8px 14px',
        fontFamily: MONO, fontSize: 14, lineHeight: 1.5, color: '#fff',
        overflow: 'hidden',
      }}>
        {children}
      </div>
    </div>
  );
};

// ─────────── Input Box (inline, part of content flow) ───────────
// Appears right after the last message. User types here.
// After typing completes (submitted), it disappears and the text becomes a message above.
interface InputBoxProps {
  text: string;
  delay?: number;
  typingSpeed?: number;
  /** Frame when "enter" is pressed — text clears, box stays empty */
  submitFrame?: number;
}

export const InputBox: React.FC<InputBoxProps> = ({
  text,
  delay = 0,
  typingSpeed = 1.5,
  submitFrame,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  if (frame < delay) return null;

  // After submit: box is empty, just show cursor
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
    <div style={{ marginTop: 4 }}>
      {/* Top separator */}
      <div style={{ height: 1, background: '#333', marginBottom: 8 }} />
      {/* Input line */}
      <div style={{ display: 'flex', alignItems: 'center', minHeight: 22 }}>
        <span style={{ color: '#555', marginRight: 6 }}>›</span>
        <span style={{ color: '#888', fontWeight: 600 }}>❯ </span>
        {!submitted && (
          <span style={{ color: '#fff' }}>{text.slice(0, chars)}</span>
        )}
        <span style={{
          display: 'inline-block', width: 2, height: 16,
          background: '#fff', marginLeft: 1, verticalAlign: 'text-bottom',
          opacity: cursorOn ? 1 : 0,
        }} />
      </div>
      {/* Bottom separator */}
      <div style={{ height: 1, background: '#333', marginTop: 8 }} />
      {/* Hint */}
      <div style={{ color: '#666', fontSize: 12, marginTop: 4 }}>
        <span style={{ color: '#888' }}>?</span> for shortcuts
      </div>
    </div>
  );
};

// ─────────── Submitted User Message "❯ text" ───────────
// After the user presses enter, the input becomes a regular message in the flow.
export const UserMessage: React.FC<{
  text: string;
  delay?: number;
}> = ({ text, delay = 0 }) => {
  const frame = useCurrentFrame();
  if (frame < delay) return null;

  return (
    <ClaudeActivity delay={delay}>
      <span style={{ color: '#888', fontWeight: 600 }}>❯ </span>
      <span style={{ color: '#fff' }}>{text}</span>
    </ClaudeActivity>
  );
};

// ─────────── Claude Code Startup Header ───────────
export const ClaudeCodeHeader: React.FC<{ delay?: number }> = ({ delay = 0 }) => {
  const frame = useCurrentFrame();
  if (frame < delay) return null;

  return (
    <div style={{ marginBottom: 8 }}>
      <div style={{ display: 'flex', gap: 14, alignItems: 'flex-start', marginBottom: 8 }}>
        <div style={{
          width: 34, height: 34, borderRadius: 3,
          background: '#e74c3c', display: 'flex', alignItems: 'center',
          justifyContent: 'center', fontSize: 18, flexShrink: 0, marginTop: 2,
          color: '#fff', fontWeight: 700,
        }}>
          {'>_'}
        </div>
        <div>
          <div>
            <span style={{ color: '#3fb950', fontWeight: 600 }}>Claude Code</span>
            <span style={{ color: '#888' }}> v2.1.81</span>
          </div>
          <div style={{ color: '#666', fontSize: 12 }}>
            claude-sonnet-4-6 · API · ~/project
          </div>
        </div>
      </div>
    </div>
  );
};

// ─────────── Claude's text "● text" ───────────
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
    <ClaudeActivity delay={delay} style={style}>
      <span style={{ color: '#fff' }}>● </span>
      <span style={{ color: '#ddd' }}>{words.slice(0, wordsToShow).join(' ')}</span>
    </ClaudeActivity>
  );
};

// ─────────── Tool Call "● ToolName(args)" ───────────
export const ToolCall: React.FC<{
  tool: string;
  args: string;
  delay?: number;
  style?: CSSProperties;
}> = ({ tool, args, delay = 0, style }) => {
  const frame = useCurrentFrame();
  if (frame < delay) return null;

  return (
    <ClaudeActivity delay={delay} style={{ marginTop: 2, marginBottom: 2, ...style }}>
      <span style={{ color: '#3fb950' }}>● </span>
      <span style={{ color: '#3fb950', fontWeight: 600 }}>{tool}</span>
      <span style={{ color: '#888' }}>({args})</span>
    </ClaudeActivity>
  );
};

// ─────────── Tool Result "└ result" ───────────
export const ToolResult: React.FC<{
  text: string;
  delay?: number;
  color?: string;
  style?: CSSProperties;
}> = ({ text, delay = 0, color = '#888', style }) => {
  const frame = useCurrentFrame();
  if (frame < delay) return null;

  return (
    <ClaudeActivity delay={delay} style={{ paddingLeft: 16, ...style }}>
      <span style={{ color: '#555' }}>└ </span>
      <span style={{ color }}>{text}</span>
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
      fontSize: 12, marginLeft: 12,
    }}>
      <span style={{ color: cfg.fg }}>{cfg.icon} [{cfg.label}]</span>
      <span style={{ color: '#888' }}>{text}</span>
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
      <span style={{ color: '#3fb950' }}>❯❯</span>
      <span style={{ color: '#e5c07b' }}> ~ </span>
      <span style={{ color: '#fff' }}>{command.slice(0, chars)}</span>
      {(!done || cursorOn) && (
        <span style={{
          display: 'inline-block', width: 7, height: 15,
          background: '#fff', verticalAlign: 'text-bottom', marginLeft: 1,
          opacity: !done ? 1 : cursorOn ? 0.7 : 0,
        }} />
      )}
    </div>
  );
};
