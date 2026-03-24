import React from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from 'remotion';
import { MacDesktop } from '../components/MacDesktop';
import { Transition3D } from '../components/Transition3D';
import { ClaudeTerminal, ClaudeActivity } from '../components/ClaudeCodeUI';
import { CharGradientLine } from '../components/GradientText';
import { Camera, CameraKeyframe } from '../components/Camera';
import { BARK_ASCII, COLORS, SCENE_DURATIONS } from '../theme';

const CHECK_ITEMS = ['jq', 'claude CLI'];
const INSTALL_ITEMS = ['bark.sh', 'bark-ctl.sh', 'bark.conf'];
const HOW_IT_WORKS = [
  { label: 'Read-only', desc: 'Read / Grep / Glob', arrow: 'Allow', color: COLORS.low },
  { label: 'Bash', desc: 'All commands', arrow: 'AI assess (~7s)', color: COLORS.c2 },
  { label: 'Danger', desc: 'rm -rf / force push', arrow: 'Notify + confirm', color: COLORS.high },
];

const CMD = 'curl -fsSL https://raw.githubusercontent.com/user/Bark/main/install.sh | bash';

// Timeline:
// 0-5:     terminal appears
// 8-48:    typing command (camera zoomed on this line)
// 48-54:   typing done, brief "enter" pause
// 54:      output starts, camera pulls back
// 54-84:   banner + checks + install + how-it-works (all fast ~1s)
// 84-150:  hold result (~2.2s)

export const S02_Install: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // --- Typing ---
  const typeStart = 8;
  const cmdElapsed = Math.max(0, frame - typeStart);
  let cmdChars = 0;
  let acc = 0;
  for (let i = 0; i < CMD.length; i++) {
    const ch = CMD[i];
    const mult = ch === ' ' ? 0.6 : ch === '/' ? 0.4 : ch === ':' ? 0.5 : 1.0;
    acc += 1 / (2.5 * mult);
    if (acc > cmdElapsed) break;
    cmdChars++;
  }
  cmdChars = Math.min(cmdChars, CMD.length);
  const typingDone = cmdChars >= CMD.length;

  // --- Output timeline ---
  const outputStart = 54;
  const bannerLines = frame >= outputStart
    ? Math.min(Math.floor((frame - outputStart) / 1) + 1, BARK_ASCII.length)
    : 0;
  const subtitleFrame = outputStart + BARK_ASCII.length + 2;
  const checkFrame = subtitleFrame + 2;
  const installFrame = checkFrame + 6;
  const completeFrame = installFrame + 8;
  const howFrame = completeFrame + 3;

  // --- Camera keyframes ---
  // Typing: zoom in on command line (top area of terminal)
  // Terminal is centered in viewport. Command line is at top of terminal.
  // To see top of terminal: move element DOWN (positive y) so viewport shows upper part.
  const cameraKF: CameraKeyframe[] = [
    { frame: 0, scale: 1.0, x: 0, y: 0 },
    // Zoom into the command line (top of terminal)
    { frame: 10, scale: 1.6, x: 0, y: 5 },
    // Hold zoomed on command line while typing
    { frame: outputStart - 4, scale: 1.6, x: 0, y: 5 },
    // After "enter": pull back to normal to see full install output
    { frame: outputStart + 8, scale: 1.1, x: 0, y: 0 },
    // Hold for the rest
  ];

  return (
    <Transition3D type="rotateIn">
      <Camera keyframes={cameraKF}>
      <MacDesktop darken={0.4}>
        <AbsoluteFill style={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
          <ClaudeTerminal width={1100} height={660} enterDelay={3}>
            {/* Prompt: just ~ not ~/project */}
            <div style={{ marginBottom: 6 }}>
              <span style={{ color: '#3fb950' }}>❯❯</span>
              <span style={{ color: '#e5c07b' }}> ~ </span>
              <span style={{ color: '#fff' }}>{CMD.slice(0, cmdChars)}</span>
              {!typingDone && (
                <span style={{
                  display: 'inline-block', width: 7, height: 15,
                  background: '#fff', verticalAlign: 'text-bottom', marginLeft: 1,
                }} />
              )}
            </div>

            {/* Banner */}
            {bannerLines > 0 && (
              <div style={{ fontSize: 16, lineHeight: 1.2, margin: '6px 0' }}>
                {BARK_ASCII.slice(0, bannerLines).map((line, i) => (
                  <CharGradientLine key={i} text={line} style={{ fontSize: 16 }} />
                ))}
              </div>
            )}
            {frame >= subtitleFrame && (
              <div style={{ color: '#888', fontStyle: 'italic', fontSize: 13, marginBottom: 8 }}>
                🐕 AI-Powered Risk Assessment for Claude Code v1.0.0
              </div>
            )}

            {/* Check environment */}
            {frame >= checkFrame && (
              <ClaudeActivity delay={checkFrame} style={{ marginBottom: 4 }}>
                <div style={{ color: COLORS.c1, fontWeight: 700 }}>▸ Check environment</div>
              </ClaudeActivity>
            )}
            {CHECK_ITEMS.map((item, i) => {
              const d = checkFrame + 2 + i * 2;
              return frame >= d ? (
                <ClaudeActivity key={item} delay={d} style={{ paddingLeft: 12 }}>
                  <span style={{ color: COLORS.low }}>✓ </span>{item}
                </ClaudeActivity>
              ) : null;
            })}

            {/* Install */}
            {frame >= installFrame && (
              <ClaudeActivity delay={installFrame} style={{ marginTop: 4, marginBottom: 3 }}>
                <div style={{ color: COLORS.c1, fontWeight: 700 }}>▸ Install components</div>
              </ClaudeActivity>
            )}
            {INSTALL_ITEMS.map((item, i) => {
              const d = installFrame + 2 + i * 2;
              return frame >= d ? (
                <ClaudeActivity key={item} delay={d} style={{ paddingLeft: 12 }}>
                  <span style={{ color: COLORS.low }}>✓ </span>{item}
                </ClaudeActivity>
              ) : null;
            })}

            {/* Complete + How it works */}
            {frame >= completeFrame && (
              <ClaudeActivity delay={completeFrame} style={{ marginTop: 6 }}>
                <span style={{ color: COLORS.low }}>✓ Installation complete!</span>
              </ClaudeActivity>
            )}
            {frame >= howFrame && (
              <ClaudeActivity delay={howFrame} style={{ fontWeight: 700, marginTop: 4 }}>
                How it works
              </ClaudeActivity>
            )}
            {HOW_IT_WORKS.map((item, i) => {
              const d = howFrame + 3 + i * 3;
              return frame >= d ? (
                <ClaudeActivity key={i} delay={d} style={{
                  paddingLeft: 12, display: 'flex', gap: 8, alignItems: 'center', fontSize: 13,
                }}>
                  <span style={{ color: item.color }}>◆</span>
                  <span style={{ color: '#888', width: 70 }}>{item.label}</span>
                  <span style={{ color: '#888' }}>{item.desc}</span>
                  <span style={{ color: '#555' }}>──▸</span>
                  <span style={{ color: item.color, fontWeight: 600 }}>{item.arrow}</span>
                </ClaudeActivity>
              ) : null;
            })}
          </ClaudeTerminal>
        </AbsoluteFill>
      </MacDesktop>
      </Camera>
    </Transition3D>
  );
};
