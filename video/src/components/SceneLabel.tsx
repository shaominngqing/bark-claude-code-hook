import React from 'react';
import { useCurrentFrame, useVideoConfig, spring, interpolate } from 'remotion';
import { FONT_MONO } from '../theme';

/**
 * Floating label on the right side of a scene, explaining what's being shown.
 */
interface SceneLabelProps {
  text: string;
  sub?: string;
  color?: string;
  delay?: number;
}

export const SceneLabel: React.FC<SceneLabelProps> = ({
  text,
  sub,
  color = '#fff',
  delay = 10,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  if (frame < delay) return null;

  const entered = spring({
    frame: frame - delay, fps,
    config: { damping: 20, mass: 0.8, stiffness: 200 },
  });
  const opacity = interpolate(entered, [0, 1], [0, 1]);
  const translateX = interpolate(entered, [0, 1], [20, 0]);

  return (
    <div style={{
      position: 'absolute',
      left: '72%',
      top: '38%',
      transform: `translateY(-50%) translateX(${translateX}px)`,
      opacity,
      textAlign: 'left',
      zIndex: 100,
      maxWidth: 280,
    }}>
      <div style={{
        fontSize: 22, fontWeight: 700, color,
        fontFamily: FONT_MONO,
        textShadow: '0 2px 12px rgba(0,0,0,0.8)',
        marginBottom: 4,
      }}>
        {text}
      </div>
      {sub && (
        <div style={{
          fontSize: 15, color: 'rgba(255,255,255,0.5)',
          fontFamily: FONT_MONO,
          textShadow: '0 1px 8px rgba(0,0,0,0.8)',
        }}>
          {sub}
        </div>
      )}
    </div>
  );
};
