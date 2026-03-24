import React, { CSSProperties } from 'react';
import { useCurrentFrame, useVideoConfig, spring, interpolate } from 'remotion';
import { FONT_SANS } from '../theme';
import { SPRING } from '../animations';

/**
 * Pixel-perfect macOS notification recreation.
 *
 * Based on real screenshot analysis:
 * - Rounded rect (14px radius), frosted glass background
 * - Left: app icon (small, rounded square)
 * - Title row: "⚠ Claude Code" or "🛡 Claude Code" bold
 * - Subtitle: e.g. "已自动放行" in bold
 * - Body: description in blue-ish gray
 * - Positioned top-right of screen
 * - Entrance: slides down from off-screen with spring bounce
 *
 * Two variants observed:
 * - Warning: ⚠ icon, yellow tint
 * - Error/Danger: shield icon, red tint
 */

type NotifVariant = 'warning' | 'danger';

interface MacNotificationProps {
  /** "Claude Code" or custom app name */
  appName?: string;
  /** Subtitle line — bold */
  subtitle: string;
  /** Body description — lighter color */
  body: string;
  variant?: NotifVariant;
  startFrame?: number;
  /** Auto-dismiss after N frames (0 = stay) */
  dismissAfter?: number;
  style?: CSSProperties;
}

export const MacNotification: React.FC<MacNotificationProps> = ({
  appName = 'Claude Code',
  subtitle,
  body,
  variant = 'warning',
  startFrame = 0,
  dismissAfter = 0,
  style,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  if (frame < startFrame) return null;

  const elapsed = frame - startFrame;

  // Entrance spring — bouncy slide down
  const enterSpring = spring({
    frame: elapsed,
    fps,
    config: { damping: 14, mass: 0.7, stiffness: 220 },
  });
  const enterY = interpolate(enterSpring, [0, 1], [-100, 0]);
  const enterOpacity = interpolate(enterSpring, [0, 1], [0, 1]);
  const enterScale = interpolate(enterSpring, [0, 1], [0.92, 1]);

  // Exit (if dismissAfter > 0)
  let exitOpacity = 1;
  let exitY = 0;
  if (dismissAfter > 0 && elapsed > dismissAfter) {
    const exitElapsed = elapsed - dismissAfter;
    exitOpacity = interpolate(exitElapsed, [0, 12], [1, 0], {
      extrapolateRight: 'clamp',
    });
    exitY = interpolate(exitElapsed, [0, 12], [0, -30], {
      extrapolateRight: 'clamp',
    });
  }

  const iconConfig = {
    warning: { emoji: '⚠️', bg: '#f5a623' },
    danger: { emoji: '🛡️', bg: '#4a90d9' },
  };
  const ic = iconConfig[variant];

  return (
    <div
      style={{
        position: 'absolute',
        top: '8%',
        left: '76%',
        zIndex: 200,
        opacity: enterOpacity * exitOpacity,
        transform: `translateY(${enterY + exitY}px) scale(${enterScale})`,
        ...style,
      }}
    >
      <div
        style={{
          width: 350,
          // Frosted glass effect — matches macOS notification
          background: 'rgba(240, 240, 240, 0.75)',
          backdropFilter: 'blur(50px) saturate(1.8)',
          WebkitBackdropFilter: 'blur(50px) saturate(1.8)',
          borderRadius: 14,
          padding: '12px 14px',
          display: 'flex',
          alignItems: 'flex-start',
          gap: 10,
          // Subtle border like real macOS
          border: '0.5px solid rgba(0,0,0,0.08)',
          boxShadow: `
            0 8px 32px rgba(0,0,0,0.12),
            0 2px 8px rgba(0,0,0,0.08),
            0 0 0 0.5px rgba(0,0,0,0.05)
          `,
          fontFamily: FONT_SANS,
        }}
      >
        {/* App icon — rounded square */}
        <div
          style={{
            width: 36,
            height: 36,
            borderRadius: 8,
            background: `linear-gradient(135deg, ${ic.bg}dd, ${ic.bg}99)`,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 18,
            flexShrink: 0,
            boxShadow: '0 1px 3px rgba(0,0,0,0.15)',
          }}
        >
          {ic.emoji}
        </div>

        {/* Text content */}
        <div style={{ flex: 1, minWidth: 0 }}>
          {/* App name */}
          <div style={{
            fontSize: 13,
            fontWeight: 600,
            color: '#1a1a1a',
            marginBottom: 1,
            display: 'flex',
            alignItems: 'center',
            gap: 4,
          }}>
            {appName}
          </div>
          {/* Subtitle — bold */}
          <div style={{
            fontSize: 13,
            fontWeight: 600,
            color: '#333',
            marginBottom: 2,
          }}>
            {subtitle}
          </div>
          {/* Body — lighter, can be multi-line */}
          <div style={{
            fontSize: 12,
            color: '#5a6a82',
            lineHeight: 1.35,
            overflow: 'hidden',
            display: '-webkit-box',
            WebkitLineClamp: 2,
            WebkitBoxOrient: 'vertical',
          }}>
            {body}
          </div>
        </div>
      </div>
    </div>
  );
};
