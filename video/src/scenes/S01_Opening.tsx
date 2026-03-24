import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig, spring, interpolate } from 'remotion';
import { MacDesktop } from '../components/MacDesktop';
import { Transition3D } from '../components/Transition3D';
import { CharGradientLine } from '../components/GradientText';
import { Camera, cameraOpening } from '../components/Camera';
import { BARK_ASCII, COLORS, FONT_MONO, GRADIENT_CSS, SCENE_DURATIONS } from '../theme';
import { springIn } from '../animations';

export const S01_Opening: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Staggered ASCII lines
  const subtitleDelay = BARK_ASCII.length * 7 + 18;
  const subtitleSpring = springIn(frame, fps, subtitleDelay, 'gentle');
  const subtitleY = interpolate(subtitleSpring, [0, 1], [20, 0]);
  const subtitleOpacity = interpolate(subtitleSpring, [0, 1], [0, 1]);

  return (
    <Transition3D type="pushIn">
      <Camera keyframes={cameraOpening(SCENE_DURATIONS.opening)}>
      <MacDesktop darken={0.55}>
        <AbsoluteFill style={{
          display: 'flex', flexDirection: 'column',
          alignItems: 'center', justifyContent: 'center',
        }}>
          {/* ASCII Art */}
          <div style={{ fontFamily: FONT_MONO, fontSize: 42, lineHeight: 1.25, marginBottom: 24 }}>
            {BARK_ASCII.map((line, i) => {
              const lineDelay = i * 7 + 8;
              const ls = springIn(frame, fps, lineDelay, 'snappy');
              const lo = interpolate(ls, [0, 1], [0, 1]);
              const ly = interpolate(ls, [0, 1], [15, 0]);
              return (
                <div key={i} style={{ opacity: lo, transform: `translateY(${ly}px)` }}>
                  <CharGradientLine
                    text={line}
                    colors={['#38bdf8', '#60a5fa', '#818cf8', '#a78bfa', '#c084fc']}
                  />
                </div>
              );
            })}
          </div>

          {/* Subtitle */}
          <div style={{
            opacity: subtitleOpacity,
            transform: `translateY(${subtitleY}px)`,
            fontFamily: FONT_MONO, fontSize: 21,
            color: 'rgba(255,255,255,0.7)', fontStyle: 'italic', textAlign: 'center',
            textShadow: '0 2px 20px rgba(0,0,0,0.5)',
          }}>
            🐕 AI-Powered Risk Assessment for Claude Code
            <span style={{
              marginLeft: 16, background: GRADIENT_CSS,
              WebkitBackgroundClip: 'text', WebkitTextFillColor: 'transparent',
              backgroundClip: 'text', fontWeight: 700, fontStyle: 'normal',
            }}>
              v1.0.0
            </span>
          </div>
        </AbsoluteFill>
      </MacDesktop>
      </Camera>
    </Transition3D>
  );
};
