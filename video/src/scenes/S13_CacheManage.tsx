import React from 'react';
import { AbsoluteFill, useCurrentFrame } from 'remotion';
import { MacDesktop } from '../components/MacDesktop';
import { Transition3D } from '../components/Transition3D';
import { Camera, cameraZoomToContent } from '../components/Camera';
import { ClaudeTerminal, ClaudeActivity, ShellPrompt } from '../components/ClaudeCodeUI';
import { SceneLabel } from '../components/SceneLabel';
import { GradientText } from '../components/GradientText';
import { COLORS, SCENE_DURATIONS } from '../theme';

const ENTRIES = [
  { reason: 'git commit 本地操作', age: '2h ago' },
  { reason: '只读目录列出', age: '5h ago' },
  { reason: 'git push 远程推送', age: '1d ago' },
];

export const S13_CacheManage: React.FC = () => {
  const frame = useCurrentFrame();

  return (
    <Transition3D type="fadeScale">
      <Camera keyframes={cameraZoomToContent(SCENE_DURATIONS.cacheManage)}>
      <MacDesktop darken={0.4}>
        <SceneLabel text="📦 Cache Management" sub="缓存管理 · bark cache" color={COLORS.c2} delay={8} />
        <AbsoluteFill style={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
          <ClaudeTerminal width={1000} height={420} enterDelay={3} title="bark cache">
            <ShellPrompt command="bark cache" delay={3} typingSpeed={4.0} />
            {frame >= 10 && (
              <ClaudeActivity delay={10} style={{ marginBottom: 6 }}>
                <GradientText style={{ fontWeight: 700, fontSize: 16 }}>Cache</GradientText>
              </ClaudeActivity>
            )}
            {frame >= 13 && (
              <ClaudeActivity delay={13}>
                <span style={{ color: '#888' }}>Entries  </span>
                <span style={{ color: COLORS.c2, fontWeight: 700 }}>31</span>
                <span style={{ color: '#888', marginLeft: 24 }}>Size  </span>
                <span style={{ color: COLORS.c2 }}>12K</span>
              </ClaudeActivity>
            )}
            {frame >= 16 && (
              <ClaudeActivity delay={16} style={{ fontWeight: 700, marginTop: 6, marginBottom: 3 }}>Recent</ClaudeActivity>
            )}
            {ENTRIES.map((e, i) => {
              const d = 19 + i * 4;
              return frame >= d ? (
                <ClaudeActivity key={i} delay={d} style={{ paddingLeft: 8, marginBottom: 1 }}>
                  <span style={{ color: COLORS.c2 }}>◆ </span>
                  <span style={{ color: '#ccc' }}>{e.reason}</span>
                  <span style={{ color: '#666' }}> ({e.age})</span>
                </ClaudeActivity>
              ) : null;
            })}
            {frame >= 38 && (
              <div style={{ marginTop: 10 }}>
                <ShellPrompt command="bark cache clear" delay={38} typingSpeed={4.0} />
              </div>
            )}
            {frame >= 48 && (
              <ClaudeActivity delay={48}>
                <span style={{ color: COLORS.low }}>✅ Cache cleared</span>
              </ClaudeActivity>
            )}
          </ClaudeTerminal>
        </AbsoluteFill>
      </MacDesktop>
      </Camera>
    </Transition3D>
  );
};
