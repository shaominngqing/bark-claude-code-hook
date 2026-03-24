import React, { CSSProperties } from 'react';
import { useCurrentFrame, useVideoConfig } from 'remotion';
import { COLORS, FONT_MONO } from '../theme';
import { cursorBlink } from '../animations';

interface TypewriterProps {
  text: string;
  startFrame?: number;
  /** Chars per frame (higher = faster) */
  speed?: number;
  showCursor?: boolean;
  style?: CSSProperties;
  color?: string;
}

export const Typewriter: React.FC<TypewriterProps> = ({
  text,
  startFrame = 0,
  speed = 1.8,
  showCursor = true,
  style,
  color,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const elapsed = frame - startFrame;
  if (elapsed < 0) return null;

  // Variable-speed typing for realism
  let charsToShow = 0;
  let accum = 0;
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    // Slow on punctuation/spaces, fast on alphanumeric bursts
    const mult = ch === ' ' ? 0.6
      : ch === '/' || ch === '\\' ? 0.4
      : ch === '.' || ch === ':' ? 0.5
      : ch === '-' ? 0.55
      : 1.0;
    accum += 1 / (speed * mult);
    if (accum > elapsed) break;
    charsToShow++;
  }
  charsToShow = Math.min(charsToShow, text.length);

  const done = charsToShow >= text.length;
  const blinkOn = cursorBlink(frame, fps);

  return (
    <span style={{ fontFamily: FONT_MONO, color: color || COLORS.termText, ...style }}>
      {text.slice(0, charsToShow)}
      {showCursor && (
        <span
          style={{
            display: 'inline-block',
            width: 8,
            height: '1.1em',
            background: COLORS.termText,
            marginLeft: 1,
            verticalAlign: 'text-bottom',
            opacity: !done ? 1 : blinkOn ? 0.8 : 0,
          }}
        />
      )}
    </span>
  );
};

// Multi-line reveal with staggered entrance
interface MultiLineTypewriterProps {
  lines: { text: string; color?: string; prefix?: string; prefixColor?: string }[];
  startFrame?: number;
  framesPerLine?: number;
  style?: CSSProperties;
}

export const MultiLineTypewriter: React.FC<MultiLineTypewriterProps> = ({
  lines,
  startFrame = 0,
  framesPerLine = 20,
  style,
}) => {
  const frame = useCurrentFrame();
  const elapsed = frame - startFrame;
  if (elapsed < 0) return null;

  const visibleCount = Math.min(Math.floor(elapsed / framesPerLine) + 1, lines.length);

  return (
    <div style={style}>
      {lines.slice(0, visibleCount).map((line, i) => {
        const lineStart = i * framesPerLine;
        const lineElapsed = elapsed - lineStart;
        const charsToShow = Math.min(Math.floor(lineElapsed * 1.5), line.text.length);

        return (
          <div key={i} style={{ whiteSpace: 'pre' }}>
            {line.prefix && (
              <span style={{ color: line.prefixColor || COLORS.termDim }}>{line.prefix}</span>
            )}
            <span style={{ color: line.color || COLORS.termText }}>
              {line.text.slice(0, charsToShow)}
            </span>
          </div>
        );
      })}
    </div>
  );
};
