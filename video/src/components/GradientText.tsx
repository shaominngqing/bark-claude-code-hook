import React, { CSSProperties } from 'react';
import { COLORS, GRADIENT_CYCLE } from '../theme';

interface GradientTextProps {
  children: React.ReactNode;
  style?: CSSProperties;
  from?: string;
  via?: string;
  to?: string;
}

export const GradientText: React.FC<GradientTextProps> = ({
  children,
  style,
  from = COLORS.gradientStart,
  via = COLORS.gradientMid,
  to = COLORS.gradientEnd,
}) => {
  return (
    <span
      style={{
        background: `linear-gradient(135deg, ${from}, ${via}, ${to})`,
        WebkitBackgroundClip: 'text',
        WebkitTextFillColor: 'transparent',
        backgroundClip: 'text',
        ...style,
      }}
    >
      {children}
    </span>
  );
};

// Per-character gradient — matches install.sh _gradient_line() with colors 39-44
interface CharGradientLineProps {
  text: string;
  colors?: string[];
  style?: CSSProperties;
}

export const CharGradientLine: React.FC<CharGradientLineProps> = ({
  text,
  colors = GRADIENT_CYCLE,
  style,
}) => {
  const chars = text.split('');
  return (
    <div style={{ whiteSpace: 'pre', ...style }}>
      {chars.map((ch, i) => (
        <span
          key={i}
          style={{
            color: colors[i % colors.length],
            fontWeight: 700,
          }}
        >
          {ch}
        </span>
      ))}
    </div>
  );
};
