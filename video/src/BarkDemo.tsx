import React from 'react';
import { AbsoluteFill, Sequence } from 'remotion';
import { FPS, SCENE_DURATIONS, COLORS } from './theme';

import { S01_Opening } from './scenes/S01_Opening';
import { S02_Install } from './scenes/S02_Install';
import { S03_Workflow } from './scenes/S03_Workflow';
import { S12_Stats } from './scenes/S12_Stats';
import { S13_CacheManage } from './scenes/S13_CacheManage';
import { S14_LogView } from './scenes/S14_LogView';
import { S15_OnOff } from './scenes/S15_OnOff';
import { S17_Help } from './scenes/S17_Help';

const scenes = [
  { Component: S01_Opening, duration: SCENE_DURATIONS.opening },
  { Component: S02_Install, duration: SCENE_DURATIONS.install },
  { Component: S03_Workflow, duration: SCENE_DURATIONS.workflow },
  { Component: S12_Stats, duration: SCENE_DURATIONS.stats },
  { Component: S13_CacheManage, duration: SCENE_DURATIONS.cacheManage },
  { Component: S14_LogView, duration: SCENE_DURATIONS.logView },
  { Component: S15_OnOff, duration: SCENE_DURATIONS.onOff },
  { Component: S17_Help, duration: SCENE_DURATIONS.help },
];

export const BarkDemo: React.FC = () => {
  let offset = 0;

  return (
    <AbsoluteFill style={{ background: COLORS.termBg }}>
      {scenes.map((scene, i) => {
        const from = offset;
        const durationInFrames = scene.duration * FPS;
        offset += durationInFrames;
        const { Component } = scene;

        return (
          <Sequence key={i} from={from} durationInFrames={durationInFrames}>
            <AbsoluteFill>
              <Component />
            </AbsoluteFill>
          </Sequence>
        );
      })}
    </AbsoluteFill>
  );
};
