import React, { CSSProperties } from 'react';
import { useCurrentFrame, useVideoConfig, interpolate } from 'remotion';
import { COLORS, FONT_MONO, SHADOWS } from '../theme';
import { springIn, pulseGlow } from '../animations';

type RiskLevel = 'low' | 'medium' | 'high';

const RISK_CONFIG: Record<RiskLevel, { color: string; bg: string; icon: string; label: string; glow: string }> = {
  low: { color: COLORS.low, bg: COLORS.lowBg, icon: '\u2713', label: 'Low', glow: SHADOWS.glowLow },
  medium: { color: COLORS.medium, bg: COLORS.mediumBg, icon: '\u26A0', label: 'Medium', glow: SHADOWS.glowMedium },
  high: { color: COLORS.high, bg: COLORS.highBg, icon: '\uD83D\uDEA8', label: 'High', glow: SHADOWS.glowHigh },
};

interface RiskTagProps {
  level: RiskLevel;
  text: string;
  startFrame?: number;
  style?: CSSProperties;
  pulse?: boolean;
}

export const RiskTag: React.FC<RiskTagProps> = ({
  level,
  text,
  startFrame = 0,
  style,
  pulse = false,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  if (frame < startFrame) return null;

  const config = RISK_CONFIG[level];

  // Spring entrance — pops in from the right
  const entered = springIn(frame, fps, startFrame, level === 'high' ? 'bouncy' : 'snappy');
  const opacity = interpolate(entered, [0, 1], [0, 1]);
  const translateX = interpolate(entered, [0, 1], [30, 0]);
  const scale = interpolate(entered, [0, 1], [0.8, 1]);

  // Pulse glow for high risk
  const glowIntensity = (pulse || level === 'high')
    ? pulseGlow(frame, startFrame, 0.2, 0.4)
    : 1;

  return (
    <span
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        padding: '5px 14px',
        borderRadius: 8,
        background: config.bg,
        color: config.color,
        fontSize: 14,
        fontWeight: 600,
        fontFamily: FONT_MONO,
        gap: 6,
        opacity,
        transform: `translateX(${translateX}px) scale(${scale})`,
        border: `1px solid ${config.color}44`,
        boxShadow: level !== 'low'
          ? config.glow.replace(/[0-9.]+\)$/, `${0.15 * glowIntensity})`)
          : 'none',
        ...style,
      }}
    >
      <span style={{ fontSize: level === 'high' ? 13 : 14 }}>{config.icon}</span>
      <span>[{config.label}]</span>
      <span style={{ color: COLORS.termText, fontWeight: 400, opacity: 0.9 }}>{text}</span>
    </span>
  );
};
