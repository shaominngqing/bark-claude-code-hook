import React from 'react';
import { useCurrentFrame, useVideoConfig, staticFile, Img, Audio } from 'remotion';
import { springIn, fadeIn } from '../animations';
import { COLORS } from '../theme';
import { GradientText } from '../components/GradientText';

export const S05_Closing: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const logoScale = springIn(frame, fps, 5, 'gentle');
  const logoOpacity = fadeIn(frame, 0, 12);
  const pipelineOpacity = fadeIn(frame, 25, 10);
  const tagsOpacity = fadeIn(frame, 45, 10);
  const cmdOpacity = fadeIn(frame, 70, 10);
  const ghOpacity = fadeIn(frame, 95, 10);
  const sloganOpacity = fadeIn(frame, 120, 12);

  const glowSize = 350 + Math.sin(frame * 0.03) * 20;

  return (
    <>
      <Audio src={staticFile('audio/12_closing.mp3')} />
      <div style={{ width: '100%', height: '100%', background: COLORS.termBg, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', position: 'relative', overflow: 'hidden' }}>

        {/* Background glow */}
        <div style={{
          position: 'absolute',
          width: glowSize,
          height: glowSize,
          borderRadius: '50%',
          background: `radial-gradient(circle, ${COLORS.c1}12 0%, transparent 70%)`,
          filter: 'blur(50px)',
        }} />

        {/* Logo + name */}
        <div style={{ opacity: logoOpacity, transform: `scale(${logoScale})`, display: 'flex', alignItems: 'center', gap: 20, marginBottom: 28, zIndex: 1 }}>
          <Img src={staticFile('bark-logo.png')} style={{ width: 80, height: 80, borderRadius: 18 }} />
          <GradientText style={{ fontSize: 60, fontWeight: 800, letterSpacing: 4, fontFamily: "'SF Pro Display', sans-serif" }}>BARK</GradientText>
        </div>

        {/* 7-layer pipeline */}
        <div style={{ opacity: pipelineOpacity, marginBottom: 28, textAlign: 'center', zIndex: 1 }}>
          <div style={{ fontSize: 18, color: COLORS.termDim, fontFamily: "'PingFang SC', sans-serif", marginBottom: 10 }}>七层风险评估，层层把关</div>
          <div style={{ fontSize: 18, color: COLORS.c2, fontFamily: "'SF Mono', monospace", letterSpacing: 1 }}>
            Fast Rules → Custom Rules → Cache → AST → Chain → AI → Fallback
          </div>
        </div>

        {/* Tech tags */}
        <div style={{ opacity: tagsOpacity, display: 'flex', gap: 12, marginBottom: 32, zIndex: 1 }}>
          {['Rust', 'macOS', 'Linux', 'Windows', 'Open Source'].map((tag) => (
            <div key={tag} style={{
              padding: '6px 18px',
              borderRadius: 7,
              background: 'rgba(0,215,255,0.08)',
              border: `1px solid ${COLORS.c2}44`,
              color: COLORS.c2,
              fontSize: 18,
              fontFamily: "'SF Mono', monospace",
              fontWeight: 600,
            }}>{tag}</div>
          ))}
        </div>

        {/* Install command */}
        <div style={{ opacity: cmdOpacity, background: COLORS.termBgLight, border: `1px solid ${COLORS.termBorder}`, borderRadius: 12, padding: '14px 32px', marginBottom: 24, zIndex: 1 }}>
          <span style={{ color: COLORS.termDim, fontFamily: "'SF Mono', monospace", fontSize: 20 }}>$ </span>
          <span style={{ color: COLORS.c1, fontFamily: "'SF Mono', monospace", fontSize: 20 }}>curl -fsSL https://...install.sh | bash</span>
        </div>

        {/* GitHub */}
        <div style={{ opacity: ghOpacity, display: 'flex', alignItems: 'center', gap: 10, marginBottom: 32, zIndex: 1 }}>
          <span style={{ fontSize: 20, color: COLORS.termText, fontFamily: "'SF Mono', monospace" }}>github.com/shaominngqing/bark-claude-code-hook</span>
          <span style={{ fontSize: 22 }}>⭐</span>
        </div>

        {/* Slogan */}
        <div style={{ opacity: sloganOpacity, textAlign: 'center', zIndex: 1 }}>
          <div style={{ fontSize: 22, color: COLORS.termDim, fontFamily: "'SF Pro Display', sans-serif" }}>
            AI-Powered Risk Assessment for Claude Code
          </div>
        </div>
      </div>
    </>
  );
};
