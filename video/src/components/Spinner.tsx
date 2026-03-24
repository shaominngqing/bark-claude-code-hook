import React from 'react';
import { useCurrentFrame } from 'remotion';
import { COLORS, FONT_MONO } from '../theme';
import { fadeIn } from '../animations';

interface SpinnerProps {
  text?: string;
  startFrame?: number;
}

// Matches install.sh _spin() frames exactly
const SPINNER_FRAMES = [
  '\u25B8\u25B9\u25B9\u25B9\u25B9',
  '\u25B9\u25B8\u25B9\u25B9\u25B9',
  '\u25B9\u25B9\u25B8\u25B9\u25B9',
  '\u25B9\u25B9\u25B9\u25B8\u25B9',
  '\u25B9\u25B9\u25B9\u25B9\u25B8',
];

export const Spinner: React.FC<SpinnerProps> = ({ text = 'AI assessing...', startFrame = 0 }) => {
  const frame = useCurrentFrame();
  const elapsed = frame - startFrame;
  if (elapsed < 0) return null;

  // Spin at roughly 0.08s per frame (from script sleep 0.08)
  // At 30fps: 0.08s ≈ 2.4 frames per spinner step
  const idx = Math.floor(elapsed / 2.4) % SPINNER_FRAMES.length;
  const opacity = fadeIn(frame, startFrame, 8);

  // Subtle glow on active arrow
  const glowPulse = 0.6 + Math.sin(elapsed * 0.3) * 0.4;

  return (
    <span style={{ fontFamily: FONT_MONO, fontSize: 16, opacity }}>
      <span style={{
        color: COLORS.c2,  // matches script: ${C2}${frames[...]}
        textShadow: `0 0 ${8 * glowPulse}px ${COLORS.c2}60`,
        letterSpacing: 1,
      }}>
        {SPINNER_FRAMES[idx]}
      </span>
      <span style={{ color: COLORS.termDim, marginLeft: 10 }}>{text}</span>
    </span>
  );
};
