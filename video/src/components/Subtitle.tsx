import React from 'react';
import { useCurrentFrame, interpolate } from 'remotion';

interface SubtitleProps {
  zh: string;
  en: string;
  startFrame: number;
  duration: number;
}

/**
 * Clean subtitle — white text with shadow/stroke, no background.
 */
export const Subtitle: React.FC<SubtitleProps> = ({ zh, en, startFrame, duration }) => {
  const frame = useCurrentFrame();

  const relFrame = frame - startFrame;
  if (relFrame < 0 || relFrame > duration) return null;

  const opacity = interpolate(
    relFrame,
    [0, 5, duration - 5, duration],
    [0, 1, 1, 0],
    { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' }
  );

  const shadow = '0 1px 3px rgba(0,0,0,0.9), 0 0 8px rgba(0,0,0,0.6), 0 0 1px rgba(0,0,0,1)';

  return (
    <div
      style={{
        position: 'absolute',
        bottom: 48,
        left: 0,
        right: 0,
        opacity,
        textAlign: 'center',
        pointerEvents: 'none',
      }}
    >
      <div
        style={{
          color: '#ffffff',
          fontSize: 26,
          fontWeight: 700,
          fontFamily: "'PingFang SC', 'Noto Sans SC', sans-serif",
          lineHeight: 1.3,
          textShadow: shadow,
          WebkitTextStroke: '0.5px rgba(0,0,0,0.3)',
        }}
      >
        {zh}
      </div>
      <div
        style={{
          color: 'rgba(255,255,255,0.8)',
          fontSize: 16,
          fontWeight: 500,
          fontFamily: "'SF Pro Display', -apple-system, sans-serif",
          marginTop: 4,
          textShadow: shadow,
        }}
      >
        {en}
      </div>
    </div>
  );
};
