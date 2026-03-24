import React from 'react';
import { AbsoluteFill, useCurrentFrame } from 'remotion';
import { MacDesktop } from '../components/MacDesktop';
import { Transition3D } from '../components/Transition3D';
import { Camera, cameraZoomToContent } from '../components/Camera';
import { ClaudeTerminal, ClaudeActivity, ShellPrompt } from '../components/ClaudeCodeUI';
import { SceneLabel } from '../components/SceneLabel';
import { GradientText } from '../components/GradientText';
import { COLORS, SCENE_DURATIONS } from '../theme';

const LOGS = [
  { level: 'LOW', color: COLORS.low, time: '14:23', source: 'FAST', tool: 'Read', cmd: 'src/main.ts' },
  { level: 'LOW', color: COLORS.low, time: '14:23', source: 'FAST', tool: 'Grep', cmd: '"fn" src/' },
  { level: 'MED', color: COLORS.medium, time: '14:24', source: 'AI', tool: 'Bash', cmd: 'git push' },
  { level: 'LOW', color: COLORS.low, time: '14:25', source: 'CACHE', tool: 'Bash', cmd: 'git commit' },
  { level: 'HIGH', color: COLORS.high, time: '14:26', source: 'AI', tool: 'Bash', cmd: 'git reset --hard' },
  { level: 'LOW', color: COLORS.low, time: '14:27', source: 'FAST', tool: 'Glob', cmd: '**/*.ts' },
];

export const S14_LogView: React.FC = () => {
  const frame = useCurrentFrame();

  return (
    <Transition3D type="rotateIn">
      <Camera keyframes={cameraZoomToContent(SCENE_DURATIONS.logView)}>
      <MacDesktop darken={0.4}>
        <SceneLabel text="📋 Log Viewer" sub="日志查看 · bark log" color={COLORS.gradientMid} delay={8} />
        <AbsoluteFill style={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
          <ClaudeTerminal width={1100} height={460} enterDelay={3} title="bark log">
            <ShellPrompt command="bark log" delay={3} typingSpeed={4.0} />
            {frame >= 10 && (
              <ClaudeActivity delay={10} style={{ marginBottom: 6 }}>
                <GradientText style={{ fontWeight: 700, fontSize: 16 }}>Last 20 entries</GradientText>
              </ClaudeActivity>
            )}
            {LOGS.map((e, i) => {
              const d = 14 + i * 4;
              return frame >= d ? (
                <ClaudeActivity key={i} delay={d} style={{
                  display: 'flex', alignItems: 'center', gap: 10,
                  borderLeft: `3px solid ${e.color}`, paddingLeft: 10,
                  marginBottom: 2, fontSize: 13,
                }}>
                  <span style={{ color: e.color, width: 36, fontWeight: 600 }}>{e.level}</span>
                  <span style={{ color: '#666', width: 36 }}>{e.time}</span>
                  <span style={{ color: COLORS.gradientMid, width: 45 }}>{e.source}</span>
                  <span style={{ color: '#ccc', width: 45 }}>{e.tool}</span>
                  <span style={{ color: '#888' }}>{e.cmd}</span>
                </ClaudeActivity>
              ) : null;
            })}
            {frame >= 44 && (
              <div style={{ marginTop: 10 }}>
                <ShellPrompt command="bark log clear" delay={44} typingSpeed={4.0} />
              </div>
            )}
            {frame >= 52 && (
              <ClaudeActivity delay={52}>
                <span style={{ color: COLORS.low }}>✅ Log cleared</span>
              </ClaudeActivity>
            )}
          </ClaudeTerminal>
        </AbsoluteFill>
      </MacDesktop>
      </Camera>
    </Transition3D>
  );
};
