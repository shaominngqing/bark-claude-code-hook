import React, { CSSProperties } from 'react';
import { useCurrentFrame, useVideoConfig, interpolate } from 'remotion';
import { COLORS, FONT_MONO } from '../theme';
import { springIn, EASE } from '../animations';

interface ProgressBarProps {
  label: string;
  value: number;
  max: number;
  color: string;
  suffix?: string;
  startFrame?: number;
  style?: CSSProperties;
}

export const ProgressBar: React.FC<ProgressBarProps> = ({
  label,
  value,
  max,
  color,
  suffix = '',
  startFrame = 0,
  style,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  if (frame < startFrame) return null;

  const elapsed = frame - startFrame;

  // Animate fill with easing
  const progress = interpolate(elapsed, [0, 35], [0, value / max], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: EASE.out,
  });
  const displayValue = Math.round(interpolate(elapsed, [0, 35], [0, value], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: EASE.out,
  }));

  // Entry fade
  const entryOpacity = interpolate(elapsed, [0, 10], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  const entryX = interpolate(elapsed, [0, 12], [-15, 0], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: EASE.out,
  });

  // Matches install.sh _abar() with 24-char width
  const totalBlocks = 24;
  const filledBlocks = Math.round(progress * totalBlocks);

  // Shine sweep effect on filled portion
  const shineSweep = elapsed > 20 ? (elapsed - 20) * 0.8 : -5;

  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 12,
        fontFamily: FONT_MONO,
        fontSize: 15,
        opacity: entryOpacity,
        transform: `translateX(${entryX}px)`,
        ...style,
      }}
    >
      <span style={{ width: 60, color, textAlign: 'right', fontWeight: 600 }}>{label}</span>
      <span style={{ letterSpacing: 1, position: 'relative' }}>
        <span style={{ color }}>{'\u2593'.repeat(filledBlocks)}</span>
        <span style={{ color: COLORS.termDim, opacity: 0.4 }}>{'\u2591'.repeat(totalBlocks - filledBlocks)}</span>
      </span>
      <span style={{ color: COLORS.termText, fontWeight: 700, minWidth: 32 }}>
        {displayValue}
      </span>
      <span style={{ color: COLORS.termDim, fontSize: 13 }}>
        ({Math.round((value / max) * 100)}%)
      </span>
      {suffix && <span style={{ color: COLORS.termDim, fontSize: 12 }}>{suffix}</span>}
    </div>
  );
};
