import React from 'react';
import { AbsoluteFill, useCurrentFrame } from 'remotion';
import { MacDesktop } from '../components/MacDesktop';
import { Transition3D } from '../components/Transition3D';
import { Camera, cameraTypeAndReveal } from '../components/Camera';
import { ClaudeTerminal, ClaudeActivity, ShellPrompt } from '../components/ClaudeCodeUI';
import { CharGradientLine } from '../components/GradientText';
import { SceneLabel } from '../components/SceneLabel';
import { BARK_ASCII_SMALL, COLORS, SCENE_DURATIONS } from '../theme';

const CMDS = [
  { cmd: 'bark off', delay: 3, resultDelay: 10, output: '● Bark disabled (takes effect in new sessions)', color: '#888' },
  { cmd: 'bark on', delay: 22, resultDelay: 29, output: '● Bark enabled (takes effect in new sessions)', color: COLORS.low },
  { cmd: 'bark version', delay: 42, resultDelay: 50, output: null, color: '' },
  { cmd: 'bark update', delay: 68, resultDelay: 76, output: 'Updating... ● Already up to date v1.0.0', color: COLORS.low },
];

export const S15_OnOff: React.FC = () => {
  const frame = useCurrentFrame();

  return (
    <Transition3D type="pushIn">
      <Camera keyframes={cameraTypeAndReveal(SCENE_DURATIONS.onOff)}>
      <MacDesktop darken={0.4}>
        <SceneLabel text="🔧 Toggle & Update" sub="开关控制 · bark on/off" color={COLORS.gradientStart} delay={8} />
        <AbsoluteFill style={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
          <ClaudeTerminal width={1000} height={500} enterDelay={3} title="bark">
            {CMDS.map((item, i) => {
              if (frame < item.delay) return null;
              return (
                <div key={i} style={{ marginBottom: 10 }}>
                  <ShellPrompt command={item.cmd} delay={item.delay} typingSpeed={4.0} />
                  {frame >= item.resultDelay && item.output && (
                    <ClaudeActivity delay={item.resultDelay} style={{ paddingLeft: 8, marginTop: 2 }}>
                      <span style={{ color: item.color }}>{item.output}</span>
                    </ClaudeActivity>
                  )}
                  {frame >= item.resultDelay && item.cmd === 'bark version' && (
                    <ClaudeActivity delay={item.resultDelay} style={{ paddingLeft: 8, marginTop: 4 }}>
                      {BARK_ASCII_SMALL.map((line, j) => (
                        <CharGradientLine key={j} text={line} style={{ fontSize: 13, lineHeight: 1.2 }} />
                      ))}
                      <div style={{ color: '#888', fontSize: 12, marginTop: 3 }}>v1.0.0</div>
                    </ClaudeActivity>
                  )}
                </div>
              );
            })}
          </ClaudeTerminal>
        </AbsoluteFill>
      </MacDesktop>
      </Camera>
    </Transition3D>
  );
};
