import React from 'react';
import { Video, staticFile } from 'remotion';

interface VideoPlayerProps {
  src?: string;
  startFrom?: number; // frame in source video (60fps)
  endAt?: number;     // frame in source video (60fps)
  style?: React.CSSProperties;
}

/**
 * Wrapper around Remotion's <Video> to play the demo recording.
 * The source video is 60fps 1080p; Remotion handles frame rate conversion.
 */
export const VideoPlayer: React.FC<VideoPlayerProps> = ({
  src = 'demo.mp4',
  startFrom,
  endAt,
  style,
}) => {
  return (
    <Video
      src={staticFile(src)}
      startFrom={startFrom}
      endAt={endAt}
      style={{
        width: '100%',
        height: '100%',
        objectFit: 'contain',
        ...style,
      }}
    />
  );
};
