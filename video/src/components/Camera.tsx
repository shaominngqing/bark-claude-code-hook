import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig, interpolate, Easing } from 'remotion';

/**
 * Camera system — smooth zoom & pan.
 *
 * Design principle: the camera should mostly be STILL.
 * It moves only at key moments (scene start, notification popup),
 * then locks in place. No constant drifting or restless movement.
 */

export interface CameraKeyframe {
  frame: number;
  scale: number;
  x: number;  // % offset from center
  y: number;  // % offset from center
}

interface CameraProps {
  children: React.ReactNode;
  keyframes: CameraKeyframe[];
}

export const Camera: React.FC<CameraProps> = ({ children, keyframes }) => {
  const frame = useCurrentFrame();

  const sorted = [...keyframes].sort((a, b) => a.frame - b.frame);

  let scale = sorted[0]?.scale ?? 1;
  let x = sorted[0]?.x ?? 0;
  let y = sorted[0]?.y ?? 0;

  if (sorted.length > 1) {
    // Find current segment
    let fromIdx = 0;
    for (let i = 0; i < sorted.length - 1; i++) {
      if (frame >= sorted[i].frame) fromIdx = i;
    }
    const toIdx = Math.min(fromIdx + 1, sorted.length - 1);
    const from = sorted[fromIdx];
    const to = sorted[toIdx];

    if (fromIdx === toIdx || frame >= to.frame) {
      // Past the last keyframe — hold still
      scale = to.scale;
      x = to.x;
      y = to.y;
    } else if (frame <= from.frame) {
      scale = from.scale;
      x = from.x;
      y = from.y;
    } else {
      const progress = interpolate(frame, [from.frame, to.frame], [0, 1], {
        extrapolateLeft: 'clamp',
        extrapolateRight: 'clamp',
        easing: Easing.bezier(0.25, 0.1, 0.25, 1.0),
      });
      scale = from.scale + (to.scale - from.scale) * progress;
      x = from.x + (to.x - from.x) * progress;
      y = from.y + (to.y - from.y) * progress;
    }
  }

  return (
    <AbsoluteFill
      style={{
        transform: `scale(${scale}) translate(${x}%, ${y}%)`,
        transformOrigin: 'center center',
      }}
    >
      {children}
    </AbsoluteFill>
  );
};

// ---- Presets ----

/**
 * Most scenes: zoom in at the start, then hold still.
 */
export function cameraZoomToContent(sceneDuration: number): CameraKeyframe[] {
  const fps = 30;
  return [
    { frame: 0, scale: 1.0, x: 0, y: 0 },
    { frame: Math.round(fps * 0.8), scale: 1.15, x: 0, y: -1 },
    // Hold here for the rest of the scene
  ];
}

/**
 * Typing scenes: zoom to prompt area, then hold.
 */
export function cameraTypeAndReveal(sceneDuration: number): CameraKeyframe[] {
  const fps = 30;
  return [
    { frame: 0, scale: 1.0, x: 0, y: 0 },
    { frame: Math.round(fps * 0.8), scale: 1.2, x: -1, y: -2 },
    // Hold
  ];
}

/**
 * Stats/dashboard: slow gentle zoom in, one move only.
 */
export function cameraSteadyZoom(sceneDuration: number): CameraKeyframe[] {
  const fps = 30;
  const total = sceneDuration * fps;
  return [
    { frame: 0, scale: 1.05, x: 0, y: 0 },
    { frame: total, scale: 1.2, x: 0, y: -1 },
    // Single slow zoom over entire scene
  ];
}

/**
 * Side-by-side: start on left panel, pan to right panel, pull back.
 * translate moves ELEMENT: +x = element right = viewport sees LEFT side
 */
export function cameraPanLeftRight(sceneDuration: number): CameraKeyframe[] {
  const fps = 30;
  const total = sceneDuration * fps;
  return [
    { frame: 0, scale: 1.15, x: 4, y: 0 },              // Element right → viewport sees left panel
    { frame: Math.round(total * 0.45), scale: 1.15, x: -4, y: 0 }, // Element left → viewport sees right panel
    { frame: Math.round(total * 0.75), scale: 1.05, x: 0, y: 0 },  // Pull back center
  ];
}

/**
 * Opening: slow pull-out reveal.
 */
export function cameraOpening(sceneDuration: number): CameraKeyframe[] {
  const fps = 30;
  const total = sceneDuration * fps;
  return [
    { frame: 0, scale: 1.15, x: 0, y: 0 },
    { frame: total, scale: 1.0, x: 0, y: 0 },
  ];
}

/**
 * Notification scenes (S05, S07):
 * Start zoomed on terminal → when notification pops, pan to top-right → hold there.
 *
 * Transform logic: translate moves the ELEMENT, not the viewport.
 * To VIEW the top-right corner, we move the element LEFT and DOWN:
 *   x: negative = element moves left = viewport sees right side
 *   y: positive = element moves down = viewport sees top side
 */
export function cameraWithNotification(
  sceneDuration: number,
  notifFrame: number,
): CameraKeyframe[] {
  const fps = 30;
  return [
    // Start: zoomed on terminal, hold still
    { frame: 0, scale: 1.0, x: 0, y: 0 },
    { frame: Math.round(fps * 0.8), scale: 1.15, x: 0, y: 0 },
    // Hold here until notification appears
    { frame: notifFrame - 2, scale: 1.15, x: 0, y: 0 },
    // Pan to top-right where notification is (at top:120 right:240)
    { frame: notifFrame + 14, scale: 1.3, x: -8, y: 8 },
    // Hold on notification for the rest of the scene
  ];
}
