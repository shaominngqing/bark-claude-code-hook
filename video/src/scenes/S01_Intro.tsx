import React from 'react';
import { useCurrentFrame, useVideoConfig, staticFile, Img, Audio } from 'remotion';
import { springIn, fadeIn } from '../animations';
import { COLORS } from '../theme';
import { GradientText } from '../components/GradientText';

export const S01_Intro: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const logoScale = springIn(frame, fps, 5, 'bouncy');
  const logoOpacity = fadeIn(frame, 0, 15);
  const versionOpacity = fadeIn(frame, 20, 10);
  const tagsOpacity = fadeIn(frame, 45, 10);
  const sloganOpacity = fadeIn(frame, 75, 12);

  // Subtle background glow animation
  const glowSize = 400 + Math.sin(frame * 0.04) * 30;

  return (
    <>
      <Audio src={staticFile('audio/01_intro.mp3')} />
      <div style={{ width: '100%', height: '100%', background: COLORS.termBg, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', overflow: 'hidden', position: 'relative' }}>

        {/* Background glow */}
        <div style={{
          position: 'absolute',
          width: glowSize,
          height: glowSize,
          borderRadius: '50%',
          background: `radial-gradient(circle, ${COLORS.c1}15 0%, transparent 70%)`,
          filter: 'blur(60px)',
        }} />

        {/* Logo */}
        <div style={{ opacity: logoOpacity, transform: `scale(${logoScale})`, marginBottom: 28, zIndex: 1 }}>
          <Img src={staticFile('bark-logo.png')} style={{ width: 160, height: 160, borderRadius: 32 }} />
        </div>

        {/* BARK 2.0 */}
        <div style={{ opacity: logoOpacity, transform: `scale(${logoScale})`, display: 'flex', alignItems: 'baseline', gap: 16, marginBottom: 20, zIndex: 1 }}>
          <GradientText style={{ fontSize: 88, fontWeight: 800, fontFamily: "'SF Pro Display', sans-serif", letterSpacing: 6 }}>BARK</GradientText>
          <span style={{ fontSize: 38, fontWeight: 700, color: COLORS.c2, fontFamily: "'SF Mono', monospace" }}>2.0</span>
        </div>

        {/* Slogan */}
        <div style={{ opacity: versionOpacity, fontSize: 24, color: COLORS.termDim, fontFamily: "'SF Mono', monospace", letterSpacing: 1, marginBottom: 36, zIndex: 1 }}>
          AI-Powered Risk Assessment for Claude Code
        </div>

        {/* Feature tags */}
        <div style={{ opacity: tagsOpacity, display: 'flex', gap: 14, marginBottom: 36, zIndex: 1 }}>
          {['全平台支持', '原生通知', '菜单栏管理', '一行安装'].map((tag) => (
            <div key={tag} style={{
              padding: '8px 22px',
              borderRadius: 8,
              background: 'rgba(0,175,255,0.08)',
              border: `1px solid ${COLORS.c1}55`,
              color: COLORS.c1,
              fontSize: 20,
              fontFamily: "'PingFang SC', sans-serif",
              fontWeight: 600,
            }}>{tag}</div>
          ))}
        </div>

        {/* English subtitle */}
        <div style={{ opacity: sloganOpacity, fontSize: 20, color: COLORS.termDim, fontFamily: "'SF Pro Display', sans-serif", letterSpacing: 0.5, zIndex: 1 }}>
          Cross-platform · Native Notifications · Menu Bar Dashboard · One Line Install
        </div>
      </div>
    </>
  );
};
