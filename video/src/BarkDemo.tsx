import React from 'react';
import { AbsoluteFill, Sequence, Audio, staticFile } from 'remotion';
import { FPS, SCENE_DURATIONS, COLORS } from './theme';

import { S01_Intro } from './scenes/S01_Intro';
import { S02_Recording } from './scenes/S02_Recording';
import { S05_Closing } from './scenes/S05_Closing';

const scenes = [
  { Component: S01_Intro, duration: SCENE_DURATIONS.intro },
  { Component: S02_Recording, duration: SCENE_DURATIONS.recording },
  { Component: S05_Closing, duration: SCENE_DURATIONS.closing },
];

export const BarkDemo: React.FC = () => {
  let offset = 0;

  return (
    <AbsoluteFill style={{ background: COLORS.termBg }}>
      {/* Background music — low volume, under voiceover */}
      <Audio src={staticFile('audio/bgm.mp3')} volume={0.25} />

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
