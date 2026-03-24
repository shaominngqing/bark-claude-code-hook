import React from 'react';
import { AbsoluteFill, useCurrentFrame } from 'remotion';
import { MacDesktop } from '../components/MacDesktop';
import { Transition3D } from '../components/Transition3D';
import { Camera, cameraSteadyZoom } from '../components/Camera';
import { ClaudeTerminal, ClaudeActivity, ShellPrompt } from '../components/ClaudeCodeUI';
import { SceneLabel } from '../components/SceneLabel';
import { GradientText } from '../components/GradientText';
import { ProgressBar } from '../components/ProgressBar';
import { AnimatedNumber } from '../components/AnimatedNumber';
import { COLORS, SCENE_DURATIONS } from '../theme';

const TOTAL = 142;
const SOURCES = [
  { label: 'FAST', value: 89, color: COLORS.low, suffix: 'fast rules' },
  { label: 'CACHE', value: 31, color: COLORS.c2, suffix: 'cache hit' },
  { label: 'RULE', value: 5, color: COLORS.purple, suffix: 'custom rules' },
  { label: 'AI', value: 17, color: COLORS.orange, suffix: 'AI assessed' },
];
const RISKS = [
  { label: 'Low', value: 120, color: COLORS.low },
  { label: 'Medium', value: 18, color: COLORS.medium },
  { label: 'High', value: 4, color: COLORS.high },
];

export const S12_Stats: React.FC = () => {
  const frame = useCurrentFrame();

  return (
    <Transition3D type="pivotLeft">
      <Camera keyframes={cameraSteadyZoom(SCENE_DURATIONS.stats)}>
      <MacDesktop darken={0.4}>
        <SceneLabel text="📊 Statistics Dashboard" sub="统计仪表板 · bark stats" color={COLORS.c2} delay={8} />
        <AbsoluteFill style={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
          <ClaudeTerminal width={1200} height={700} enterDelay={3} title="bark stats">
            <ShellPrompt command="bark stats" delay={5} typingSpeed={4.0} />

            {frame >= 12 && (
              <ClaudeActivity delay={12} style={{ marginBottom: 10 }}>
                <div style={{ color: '#555' }}>╭{'─'.repeat(39)}╮</div>
                <div style={{ color: '#555', display: 'flex', alignItems: 'center' }}>
                  │  <GradientText style={{ fontWeight: 700, fontSize: 17 }}>◆ Bark Statistics</GradientText>
                  <span style={{ flex: 1 }} /><span style={{ color: '#555' }}>│</span>
                </div>
                <div style={{ color: '#555' }}>╰{'─'.repeat(39)}╯</div>
              </ClaudeActivity>
            )}

            {frame >= 18 && (
              <ClaudeActivity delay={18} style={{ marginBottom: 8 }}>
                <span style={{ color: '#888' }}>Total  </span>
                <AnimatedNumber value={TOTAL} startFrame={18} duration={20}
                  style={{ color: COLORS.c1, fontWeight: 700, fontSize: 20 }} />
              </ClaudeActivity>
            )}

            {frame >= 24 && (
              <ClaudeActivity delay={24} style={{ marginBottom: 8 }}>
                <div style={{ fontWeight: 700, marginBottom: 4 }}>Assessment Source</div>
                {SOURCES.map((s, i) => (
                  <ProgressBar key={i} label={s.label} value={s.value} max={TOTAL}
                    color={s.color} suffix={s.suffix} startFrame={24 + i * 5}
                    style={{ marginBottom: 3 }} />
                ))}
              </ClaudeActivity>
            )}

            {frame >= 50 && (
              <ClaudeActivity delay={50} style={{ marginBottom: 8 }}>
                <div style={{ fontWeight: 700, marginBottom: 4 }}>Risk Distribution</div>
                {RISKS.map((r, i) => (
                  <div key={i} style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 3, fontSize: 14 }}>
                    <span style={{ color: r.color, width: 65 }}>◉ {r.label}</span>
                    <ProgressBar label="" value={r.value} max={TOTAL} color={r.color}
                      startFrame={50 + i * 5} />
                  </div>
                ))}
              </ClaudeActivity>
            )}

            {frame >= 72 && (
              <ClaudeActivity delay={72} style={{ display: 'flex', gap: 12, fontSize: 13 }}>
                <div>
                  <div style={{ color: '#555' }}>┌{'─'.repeat(22)}┐</div>
                  <div style={{ color: '#555' }}>│ <span style={{ fontStyle: 'italic', color: '#ccc' }}>Cache Hit Rate</span>{'       '}│</div>
                  <div style={{ color: '#555' }}>│  <span style={{ color: COLORS.c1, fontWeight: 700, fontSize: 17 }}>64%</span>{'                 '}│</div>
                  <div style={{ color: '#555' }}>└{'─'.repeat(22)}┘</div>
                </div>
                <div>
                  <div style={{ color: '#555' }}>┌{'─'.repeat(22)}┐</div>
                  <div style={{ color: '#555' }}>│ <span style={{ fontStyle: 'italic', color: '#ccc' }}>High Risk Blocked</span>{'    '}│</div>
                  <div style={{ color: '#555' }}>│  <span style={{ color: COLORS.high, fontWeight: 700, fontSize: 17 }}>4</span>{'                   '}│</div>
                  <div style={{ color: '#555' }}>└{'─'.repeat(22)}┘</div>
                </div>
              </ClaudeActivity>
            )}
          </ClaudeTerminal>
        </AbsoluteFill>
      </MacDesktop>
      </Camera>
    </Transition3D>
  );
};
