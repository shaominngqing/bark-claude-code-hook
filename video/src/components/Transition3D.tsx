import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig, interpolate, spring, Easing } from 'remotion';

/**
 * 3D perspective transitions for scenes.
 *
 * Wraps children with a CSS perspective container and applies
 * rotateX/Y + translateZ + scale animations.
 */

export type TransitionType =
  | 'rotateIn'        // Rotate from side like a page flip
  | 'pushIn'          // Push from far away (z-axis zoom)
  | 'tiltUp'          // Tilt up from below
  | 'pivotLeft'       // Pivot from left edge
  | 'fadeScale';      // Simple scale + fade (for lighter transitions)

interface Transition3DProps {
  children: React.ReactNode;
  type?: TransitionType;
  /** How many frames the entrance takes */
  enterDuration?: number;
  /** How many frames the exit takes */
  exitDuration?: number;
}

export const Transition3D: React.FC<Transition3DProps> = ({
  children,
  type = 'pushIn',
  enterDuration = 18,
  exitDuration = 12,
}) => {
  const frame = useCurrentFrame();
  const { fps, durationInFrames } = useVideoConfig();

  // --- Entrance (first N frames) ---
  const enterProgress = spring({
    frame: Math.min(frame, enterDuration),
    fps,
    config: { damping: 20, mass: 1, stiffness: 140 },
  });

  // --- Exit (last N frames) ---
  const exitFrame = durationInFrames - exitDuration;
  const exitProgress = frame > exitFrame
    ? interpolate(frame, [exitFrame, durationInFrames], [0, 1], {
        extrapolateLeft: 'clamp',
        extrapolateRight: 'clamp',
        easing: Easing.bezier(0.4, 0, 1, 1),
      })
    : 0;

  // Calculate transforms based on type
  let enterTransform = '';
  let exitTransform = '';
  let enterOpacity = enterProgress;
  let exitOpacity = 1 - exitProgress;

  switch (type) {
    case 'rotateIn': {
      const rotY = interpolate(enterProgress, [0, 1], [-35, 0]);
      const tz = interpolate(enterProgress, [0, 1], [-200, 0]);
      enterTransform = `perspective(1200px) rotateY(${rotY}deg) translateZ(${tz}px)`;
      const exitRotY = interpolate(exitProgress, [0, 1], [0, 25]);
      const exitTz = interpolate(exitProgress, [0, 1], [0, -150]);
      exitTransform = `perspective(1200px) rotateY(${exitRotY}deg) translateZ(${exitTz}px)`;
      break;
    }
    case 'pushIn': {
      const scale = interpolate(enterProgress, [0, 1], [0.85, 1]);
      const tz = interpolate(enterProgress, [0, 1], [-300, 0]);
      enterTransform = `perspective(1200px) translateZ(${tz}px) scale(${scale})`;
      const exitScale = interpolate(exitProgress, [0, 1], [1, 1.1]);
      exitTransform = `scale(${exitScale})`;
      break;
    }
    case 'tiltUp': {
      const rotX = interpolate(enterProgress, [0, 1], [15, 0]);
      const ty = interpolate(enterProgress, [0, 1], [60, 0]);
      enterTransform = `perspective(1000px) rotateX(${rotX}deg) translateY(${ty}px)`;
      const exitTy = interpolate(exitProgress, [0, 1], [0, -40]);
      exitTransform = `translateY(${exitTy}px)`;
      break;
    }
    case 'pivotLeft': {
      const rotY = interpolate(enterProgress, [0, 1], [45, 0]);
      enterTransform = `perspective(1000px) rotateY(${rotY}deg)`;
      const exitRotY = interpolate(exitProgress, [0, 1], [0, -30]);
      exitTransform = `perspective(1000px) rotateY(${exitRotY}deg)`;
      break;
    }
    case 'fadeScale': {
      const scale = interpolate(enterProgress, [0, 1], [0.95, 1]);
      enterTransform = `scale(${scale})`;
      const exitScale = interpolate(exitProgress, [0, 1], [1, 0.97]);
      exitTransform = `scale(${exitScale})`;
      break;
    }
  }

  // Combine
  const isExiting = exitProgress > 0;
  const transform = isExiting ? exitTransform : enterTransform;
  const opacity = isExiting ? exitOpacity : enterOpacity;

  return (
    <AbsoluteFill
      style={{
        transform,
        opacity,
        transformOrigin: type === 'pivotLeft' ? 'left center' : 'center center',
      }}
    >
      {children}
    </AbsoluteFill>
  );
};
