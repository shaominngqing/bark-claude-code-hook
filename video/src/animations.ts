// ============================================================================
// Core Animation Utilities — Spring physics, easing, cinematic helpers
// ============================================================================
import { spring, interpolate, Easing } from 'remotion';

// --- Spring presets (matching real UI physics) ---
export const SPRING = {
  // Snappy UI element entrance
  snappy: { damping: 28, mass: 0.8, stiffness: 350 },
  // Gentle float-in
  gentle: { damping: 20, mass: 1, stiffness: 120 },
  // Bouncy notification pop
  bouncy: { damping: 12, mass: 0.6, stiffness: 280 },
  // Heavy/dramatic entrance
  heavy: { damping: 30, mass: 1.5, stiffness: 200 },
  // Wobbly shake
  wobbly: { damping: 8, mass: 0.5, stiffness: 400 },
} as const;

// --- Easing presets ---
export const EASE = {
  out: Easing.bezier(0.16, 1, 0.3, 1),       // ease-out (expressive)
  in: Easing.bezier(0.55, 0.085, 0.68, 0.53), // ease-in
  inOut: Easing.bezier(0.77, 0, 0.175, 1),    // ease-in-out (smooth)
  outBack: Easing.bezier(0.34, 1.56, 0.64, 1), // overshoot
} as const;

/**
 * Spring-based entrance value (0 → 1)
 */
export function springIn(
  frame: number,
  fps: number,
  delay: number = 0,
  config: keyof typeof SPRING = 'snappy',
) {
  if (frame < delay) return 0;
  return spring({
    frame: frame - delay,
    fps,
    config: SPRING[config],
  });
}

/**
 * Smooth fade with easing
 */
export function fadeIn(
  frame: number,
  delay: number = 0,
  duration: number = 15,
) {
  return interpolate(frame, [delay, delay + duration], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: EASE.out,
  });
}

/**
 * Slide + fade entrance
 */
export function slideIn(
  frame: number,
  delay: number = 0,
  distance: number = 40,
  duration: number = 20,
) {
  const progress = interpolate(frame, [delay, delay + duration], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: EASE.out,
  });
  return {
    opacity: progress,
    translateY: (1 - progress) * distance,
  };
}

/**
 * Typewriter timing — variable speed per character for realism
 * Returns number of chars to show
 */
export function typewriterProgress(
  frame: number,
  text: string,
  startFrame: number,
  baseSpeed: number = 1.8,  // chars per frame at base
) {
  const elapsed = frame - startFrame;
  if (elapsed < 0) return 0;
  // Add slight variance per char to feel human
  let chars = 0;
  let accum = 0;
  for (let i = 0; i < text.length; i++) {
    // Slow down on spaces/punctuation, speed up on repeated chars
    const ch = text[i];
    const speedMult = ch === ' ' ? 0.7 : ch === '/' ? 0.5 : ch === '-' ? 0.6 : 1.0;
    accum += 1 / (baseSpeed * speedMult);
    if (accum > elapsed) break;
    chars++;
  }
  return Math.min(chars, text.length);
}

/**
 * Screen shake effect — returns {x, y} offset
 */
export function screenShake(
  frame: number,
  startFrame: number,
  intensity: number = 8,
  duration: number = 20,
) {
  const elapsed = frame - startFrame;
  if (elapsed < 0 || elapsed > duration) return { x: 0, y: 0 };
  const decay = 1 - elapsed / duration;
  const freq = 2.5;
  return {
    x: Math.sin(elapsed * freq) * intensity * decay,
    y: Math.cos(elapsed * freq * 1.3) * intensity * decay * 0.6,
  };
}

/**
 * Pulse glow effect — returns opacity multiplier (1.0 base + glow)
 */
export function pulseGlow(
  frame: number,
  startFrame: number,
  speed: number = 0.15,
  intensity: number = 0.3,
) {
  const elapsed = frame - startFrame;
  if (elapsed < 0) return 1;
  return 1 + Math.sin(elapsed * speed) * intensity;
}

/**
 * Cursor blink — classic terminal 530ms on/off
 */
export function cursorBlink(frame: number, fps: number): boolean {
  const period = Math.round(fps * 0.53); // ~530ms
  return Math.floor(frame / period) % 2 === 0;
}

/**
 * Stagger delay calculator — for list item entrances
 */
export function stagger(index: number, baseDelay: number = 0, interval: number = 6): number {
  return baseDelay + index * interval;
}

/**
 * Number counter animation
 */
export function countUp(
  frame: number,
  target: number,
  startFrame: number,
  duration: number = 30,
) {
  const progress = interpolate(frame, [startFrame, startFrame + duration], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: EASE.out,
  });
  return Math.round(target * progress);
}

/**
 * Scene transition — fade to/from black
 */
export function sceneTransition(
  frame: number,
  totalFrames: number,
  fadeFrames: number = 8,
) {
  const fadeInOpacity = interpolate(frame, [0, fadeFrames], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
    easing: EASE.out,
  });
  const fadeOutOpacity = interpolate(
    frame,
    [totalFrames - fadeFrames, totalFrames],
    [1, 0],
    { extrapolateLeft: 'clamp', extrapolateRight: 'clamp', easing: EASE.in },
  );
  return Math.min(fadeInOpacity, fadeOutOpacity);
}
