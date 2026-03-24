import React, { CSSProperties } from 'react';
import { AbsoluteFill, useCurrentFrame, useVideoConfig, interpolate, spring } from 'remotion';
import { MacDesktop } from '../components/MacDesktop';
import { Transition3D } from '../components/Transition3D';
import { ClaudeTerminal, ClaudeCodeHeader, InputBox, UserMessage, ClaudeResponse, ToolCall, BarkResult, ClaudeActivity } from '../components/ClaudeCodeUI';
import { Spinner } from '../components/Spinner';
import { MacNotification } from '../components/MacNotification';
import { Camera, CameraKeyframe, cameraWithNotification } from '../components/Camera';
import { COLORS, SHADOWS, SCENE_DURATIONS, FONT_MONO } from '../theme';
import { screenShake } from '../animations';

/**
 * Scene 3: Complete Claude Code Workflow Demo (27 seconds = 810 frames)
 *
 * One continuous Claude Code session showing ALL risk levels:
 *
 * Phase 1 (0-6s):   Read-only tools — instant green pass
 * Phase 2 (6-11s):  Normal edit vs sensitive file
 * Phase 3 (11-17s): Bash first-time AI assessment + notification
 * Phase 4 (17-21s): Cache hit — same pattern, zero delay
 * Phase 5 (21-27s): High risk — blocked + danger notification + confirm
 *
 * Right side: floating label showing current phase
 */

const FPS = 30;

// ---- Phase timelines (in frames) ----
// 30s = 900 frames. Spread out with breathing room.

// Phase 1: Read-only (0-170) ~5.7s
const P1_START = 0;
const P1_HEADER = 5;
const P1_INPUT = 14;
const P1_SUBMIT = 38;
const P1_RESPONSE = 44;
const P1_TOOLS = [
  { tool: 'Read', args: 'src/main.ts', f: 60 },
  { tool: 'Grep', args: '"handleAuth" src/', f: 85 },
  { tool: 'Glob', args: '**/*.ts', f: 110 },
];

// Phase 2: File edit (180-340) ~5.3s
const P2_START = 180;
const P2_RESPONSE = 196;
const P2_EDIT = 215;
const P2_EDIT_RESULT = 222;
const P2_WRITE = 250;
const P2_SPINNER_START = 260;
const P2_SPINNER_END = 300;

// Phase 3: First git commit — AI assess (350-540) ~6.3s
const P3_START = 350;
const P3_RESPONSE = 366;
const P3_BASH = 385;
const P3_SPINNER_START = 393;
const P3_SPINNER_END = 460;
const P3_NOTIF = P3_SPINNER_END + 12;

// Phase 4: Second git commit — cache hit (550-660) ~3.7s
const P4_START = 550;
const P4_RESPONSE = 564;
const P4_BASH = 580;
const P4_RESULT = 584;
const P4_NOTIF = 590;

// Phase 5: High risk (670-900) ~7.7s
const P5_START = 670;
const P5_RESPONSE = 686;
const P5_BASH = 705;
const P5_ALERT = 718;
const P5_NOTIF = 735;
const P5_CONFIRM = 775;

// ---- Phase label component ----
const PhaseLabel: React.FC<{
  text: string;
  sub: string;
  color: string;
  startFrame: number;
  endFrame: number;
}> = ({ text, sub, color, startFrame, endFrame }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  if (frame < startFrame || frame > endFrame) return null;

  const entered = spring({
    frame: Math.max(0, frame - startFrame),
    fps,
    config: { damping: 20, mass: 0.8, stiffness: 200 },
  });
  const opacity = interpolate(entered, [0, 1], [0, 1]);
  const translateX = interpolate(entered, [0, 1], [30, 0]);

  // Fade out near end
  const fadeOut = frame > endFrame - 15
    ? interpolate(frame, [endFrame - 15, endFrame], [1, 0], { extrapolateLeft: 'clamp', extrapolateRight: 'clamp' })
    : 1;

  return (
    <div style={{
      position: 'absolute',
      left: '72%',
      top: '38%',
      transform: `translateY(-50%) translateX(${translateX}px)`,
      opacity: opacity * fadeOut,
      textAlign: 'left',
      zIndex: 100,
      maxWidth: 280,
    }}>
      <div style={{
        fontSize: 22,
        fontWeight: 700,
        color,
        fontFamily: FONT_MONO,
        textShadow: '0 2px 12px rgba(0,0,0,0.8)',
        marginBottom: 5,
      }}>
        {text}
      </div>
      <div style={{
        fontSize: 15,
        color: 'rgba(255,255,255,0.5)',
        fontFamily: FONT_MONO,
        textShadow: '0 1px 8px rgba(0,0,0,0.8)',
      }}>
        {sub}
      </div>
    </div>
  );
};

export const S03_Workflow: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  // Screen shake for high risk
  const shake = screenShake(frame, P5_ALERT, 8, 20);

  // Red flash for high risk
  const flashIntensity = frame >= P5_ALERT && frame < P5_ALERT + 20
    ? Math.max(0, Math.sin((frame - P5_ALERT) * 0.5) * 0.15 * (1 - (frame - P5_ALERT) / 20))
    : 0;

  // Camera: mostly still, move to notification areas when needed
  // Camera: notification is at left:58% top:8%, labels at left:72%
  // Scale 1.15 = safe zone is roughly center 87% of frame
  // Only pan slightly when notification appears — it's close to center
  const cameraKF: CameraKeyframe[] = [
    // Phase 1-2: centered on terminal
    { frame: 0, scale: 1.0, x: 0, y: 0 },
    { frame: 10, scale: 1.1, x: 0, y: 0 },

    // Phase 3: notification at left:76% top:8%
    // scale 1.4 → viewport sees center 71%. Notif at 76% is outside.
    // translate(-18%, 14%) moves element far left+down → viewport sees top-right
    // Show top-right 1/4 of the ORIGINAL frame (not beyond it).
    // scale 2.0 = viewport shows exactly 1/2 width × 1/2 height = 1/4 of frame.
    // translate(-25%, 25%) centers viewport on the top-right quarter.
    // (element left 25% → viewport sees right half, element down 25% → viewport sees top half)

    // Phase 3: first AI notification — hold 700ms (21 frames)
    { frame: P3_NOTIF - 2, scale: 1.1, x: 0, y: 0 },
    { frame: P3_NOTIF + 14, scale: 2.0, x: -25, y: 25 },
    { frame: P3_NOTIF + 35, scale: 2.0, x: -25, y: 25 },  // hold 21f = 700ms
    { frame: P3_NOTIF + 49, scale: 1.1, x: 0, y: 0 },

    // Phase 4: cache notification — hold 400ms (12 frames)
    { frame: P4_NOTIF - 2, scale: 1.1, x: 0, y: 0 },
    { frame: P4_NOTIF + 10, scale: 2.0, x: -25, y: 25 },
    { frame: P4_NOTIF + 22, scale: 2.0, x: -25, y: 25 },  // hold 12f = 400ms
    { frame: P4_NOTIF + 36, scale: 1.1, x: 0, y: 0 },

    // Phase 5: danger notification — hold 500ms (15 frames)
    { frame: P5_NOTIF - 2, scale: 1.1, x: 0, y: 0 },
    { frame: P5_NOTIF + 14, scale: 2.0, x: -25, y: 25 },
    { frame: P5_NOTIF + 29, scale: 2.0, x: -25, y: 25 },  // hold 15f = 500ms
    { frame: P5_NOTIF + 43, scale: 1.1, x: 0, y: 0 },
  ];

  // How many terminal lines to show (scroll effect — hide old ones)
  // Terminal content grows, we simulate scrolling by limiting what's visible

  return (
    <Transition3D type="tiltUp" enterDuration={20} exitDuration={15}>
      <Camera keyframes={cameraKF}>
      <MacDesktop darken={0.4}>
        {/* Red flash overlay for high risk */}
        {flashIntensity > 0 && (
          <div style={{
            position: 'absolute', inset: 0, zIndex: 50, pointerEvents: 'none',
            background: `radial-gradient(ellipse at center, rgba(248,81,73,${flashIntensity}) 0%, transparent 70%)`,
          }} />
        )}

        {/* Phase labels — right side */}
        <PhaseLabel text="✓ Read-only → Allow" sub="只读操作直接放行 · Zero delay" color={COLORS.low}
          startFrame={P1_TOOLS[0].f} endFrame={P2_START} />
        <PhaseLabel text="⚠ Sensitive File" sub="敏感文件触发 AI 评估 · .env detected" color={COLORS.medium}
          startFrame={P2_WRITE} endFrame={P3_START} />
        <PhaseLabel text="⏵ AI First Assessment" sub="首次评估 ~7s · Analyzing semantics" color={COLORS.c2}
          startFrame={P3_BASH} endFrame={P4_START} />
        <PhaseLabel text="✓ Cache Hit → 0s" sub="缓存命中秒过 · Same pattern cached" color={COLORS.low}
          startFrame={P4_BASH} endFrame={P5_START} />
        <PhaseLabel text="🚨 High Risk → Blocked" sub="高风险拦截 · Needs confirmation" color={COLORS.high}
          startFrame={P5_BASH} endFrame={900} />

        <AbsoluteFill style={{
          display: 'flex', alignItems: 'center', justifyContent: 'center',
          transform: frame >= P5_ALERT && frame < P5_ALERT + 20
            ? `translate(${shake.x}px, ${shake.y}px)` : undefined,
        }}>
          <ClaudeTerminal width={1200} height={720} enterDelay={3}>
            {/* ═══════ Phase 1: Read-only ═══════ */}
            <ClaudeCodeHeader delay={P1_HEADER} />

            {/* After submit: user message appears as record */}
            <UserMessage text="fix the login bug and push the changes" delay={P1_SUBMIT} />

            {frame >= P1_RESPONSE && (
              <ClaudeResponse
                text="Let me look at the code to find the issue."
                delay={P1_RESPONSE} wordsPerFrame={0.5}
              />
            )}

            {P1_TOOLS.map((t, i) => (
              frame >= t.f ? (
                <div key={i} style={{ display: 'flex', alignItems: 'center', marginBottom: 2 }}>
                  <ToolCall tool={t.tool} args={t.args} delay={t.f} />
                  <BarkResult level="low" text="只读操作" delay={t.f + 6} />
                </div>
              ) : null
            ))}

            {/* ═══════ Phase 2: File edit vs sensitive ═══════ */}
            {frame >= P2_START && (
              <ClaudeResponse
                text="Found the bug. I'll fix the handler and update the env config."
                delay={P2_RESPONSE} wordsPerFrame={0.5}
                style={{ marginTop: 10 }}
              />
            )}

            {frame >= P2_EDIT && (
              <div style={{ display: 'flex', alignItems: 'center', marginBottom: 2 }}>
                <ToolCall tool="Edit" args="src/auth.ts" delay={P2_EDIT} />
                <BarkResult level="low" text="普通文件编辑" delay={P2_EDIT_RESULT} />
              </div>
            )}

            {frame >= P2_WRITE && (
              <div style={{ display: 'flex', alignItems: 'center', marginBottom: 2 }}>
                <ToolCall tool="Write" args=".env.production" delay={P2_WRITE} />
                {frame >= P2_SPINNER_START && frame < P2_SPINNER_END && (
                  <span style={{ marginLeft: 12 }}>
                    <Spinner text="AI assessing..." startFrame={P2_SPINNER_START} />
                  </span>
                )}
                {frame >= P2_SPINNER_END && (
                  <BarkResult level="medium" text="敏感环境变量文件" delay={P2_SPINNER_END} />
                )}
              </div>
            )}

            {/* ═══════ Phase 3: First git commit — AI assessment ═══════ */}
            {frame >= P3_START && (
              <ClaudeResponse
                text="Bug fixed. Let me commit the changes."
                delay={P3_RESPONSE} wordsPerFrame={0.5}
                style={{ marginTop: 10 }}
              />
            )}

            {frame >= P3_BASH && (
              <div style={{ display: 'flex', alignItems: 'center', marginBottom: 2 }}>
                <ToolCall tool="Bash" args={'git commit -m "fix: login bug"'} delay={P3_BASH} />
                {frame >= P3_SPINNER_START && frame < P3_SPINNER_END && (
                  <span style={{ marginLeft: 12 }}>
                    <Spinner text="AI assessing..." startFrame={P3_SPINNER_START} />
                  </span>
                )}
                {frame >= P3_SPINNER_END && (
                  <BarkResult level="low" text="本地只读式操作，可撤销" delay={P3_SPINNER_END} />
                )}
              </div>
            )}

            {/* ═══════ Phase 4: Second git commit — CACHE HIT ═══════ */}
            {frame >= P4_START && (
              <ClaudeResponse
                text="Also committing the config update."
                delay={P4_RESPONSE} wordsPerFrame={0.6}
                style={{ marginTop: 10 }}
              />
            )}

            {frame >= P4_BASH && (
              <div style={{ display: 'flex', alignItems: 'center', marginBottom: 2 }}>
                <ToolCall tool="Bash" args={'git commit -m "chore: update config"'} delay={P4_BASH} />
                <BarkResult level="low" text="本地只读式操作，可撤销" delay={P4_RESULT} cached />
              </div>
            )}

            {/* ═══════ Phase 5: git reset --hard — HIGH RISK ═══════ */}
            {frame >= P5_START && (
              <ClaudeResponse
                text="I need to undo some recent broken commits first."
                delay={P5_RESPONSE} wordsPerFrame={0.5}
                style={{ marginTop: 10 }}
              />
            )}

            {frame >= P5_BASH && (
              <div style={{ display: 'flex', alignItems: 'center', marginBottom: 2 }}>
                <ToolCall tool="Bash" args="git reset --hard HEAD~5" delay={P5_BASH} />
                {frame >= P5_ALERT && (
                  <BarkResult level="high" text="硬重置丢弃5个提交，不可逆丢失工作" delay={P5_ALERT} />
                )}
              </div>
            )}

            {/* High risk alert box */}
            {frame >= P5_ALERT + 12 && (
              <ClaudeActivity delay={P5_ALERT + 12} style={{
                marginTop: 10, padding: '12px 16px',
                background: 'rgba(248,81,73,0.08)', borderRadius: 6,
                border: `2px solid ${COLORS.high}`, boxShadow: SHADOWS.glowHigh,
              }}>
                <div style={{ color: COLORS.high, fontWeight: 700, fontSize: 15, marginBottom: 3 }}>
                  🚨 BLOCKED — Confirmation Required
                </div>
                <div style={{ color: '#ccc', fontSize: 13 }}>
                  硬重置丢弃5个提交，不可逆丢失工作
                </div>
              </ClaudeActivity>
            )}

            {/* Confirm prompt */}
            {frame >= P5_CONFIRM && (
              <ClaudeActivity delay={P5_CONFIRM} style={{ marginTop: 10 }}>
                <span style={{ color: COLORS.high }}>? </span>
                <span>Allow this operation? </span>
                <span style={{ color: '#888' }}>(y/N) </span>
                <span style={{
                  display: 'inline-block', width: 7, height: 15,
                  background: COLORS.high, verticalAlign: 'text-bottom',
                  opacity: Math.floor(frame / 16) % 2 === 0 ? 1 : 0,
                }} />
              </ClaudeActivity>
            )}

            {/* Input box — always at bottom of content, empties after submit */}
            <InputBox
              text="fix the login bug and push the changes"
              delay={P1_INPUT}
              typingSpeed={2.0}
              submitFrame={P1_SUBMIT}
            />
          </ClaudeTerminal>
        </AbsoluteFill>

        {/* ═══════ Notifications — real bark output ═══════ */}
        {/* Phase 3: first git commit — AI assessed */}
        <MacNotification
          subtitle="已自动放行"
          body="本地只读式操作，可撤销"
          variant="warning"
          startFrame={P3_NOTIF}
          dismissAfter={80}
        />

        {/* Phase 4: second git commit — cache hit */}
        <MacNotification
          subtitle="已自动放行"
          body="本地只读式操作，可撤销 (cached)"
          variant="warning"
          startFrame={P4_NOTIF}
          dismissAfter={60}
        />

        {/* Phase 5: git reset --hard — HIGH RISK */}
        <MacNotification
          subtitle="需要确认"
          body="硬重置丢弃5个提交，不可逆丢失工作"
          variant="danger"
          startFrame={P5_NOTIF}
        />
      </MacDesktop>
      </Camera>
    </Transition3D>
  );
};
