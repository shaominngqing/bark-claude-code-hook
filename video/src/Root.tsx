import React from 'react';
import { Composition } from 'remotion';
import { BarkDemo } from './BarkDemo';
import { FPS, WIDTH, HEIGHT, TOTAL_DURATION } from './theme';

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="BarkDemo"
        component={BarkDemo}
        durationInFrames={TOTAL_DURATION * FPS}
        fps={FPS}
        width={WIDTH}
        height={HEIGHT}
      />
    </>
  );
};
