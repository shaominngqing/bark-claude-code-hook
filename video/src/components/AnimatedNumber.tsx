import React, { CSSProperties } from 'react';
import { useCurrentFrame, interpolate } from 'remotion';
import { FONT_MONO } from '../theme';
import { EASE } from '../animations';

interface AnimatedNumberProps {
  value: number;
  startFrame?: number;
  duration?: number;
  style?: CSSProperties;
}

export const AnimatedNumber: React.FC<AnimatedNumberProps> = ({
  value,
  startFrame = 0,
  duration = 30,
  style,
}) => {
  const frame = useCurrentFrame();
  if (frame < startFrame) return <span style={style}>0</span>;

  const display = Math.round(
    interpolate(frame, [startFrame, startFrame + duration], [0, value], {
      extrapolateLeft: 'clamp',
      extrapolateRight: 'clamp',
      easing: EASE.out,
    })
  );

  return <span style={{ fontFamily: FONT_MONO, ...style }}>{display}</span>;
};
