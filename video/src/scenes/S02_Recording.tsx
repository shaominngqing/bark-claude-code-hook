import React from 'react';
import { useVideoConfig, Audio, Sequence, staticFile } from 'remotion';
import { VideoPlayer } from '../components/VideoPlayer';
import { Subtitle } from '../components/Subtitle';

export const S02_Recording: React.FC = () => {
  const { fps } = useVideoConfig();

  // Convert Remotion absolute seconds to local frames (scene starts at Remotion 6s)
  const at = (remotionSec: number) => Math.round((remotionSec - 6) * fps);

  return (
    <div style={{ width: '100%', height: '100%', background: '#0d1117' }}>
      <VideoPlayer />

      {/* === Voiceover === */}
      {/* 0:07-0:12 安装 */}
      <Sequence from={at(7)}><Audio src={staticFile('audio/02_install.mp3')} /></Sequence>
      {/* 0:16-0:22 菜单栏 */}
      <Sequence from={at(16)}><Audio src={staticFile('audio/03_menubar.mp3')} /></Sequence>
      {/* 0:24-0:26 进入 Claude */}
      <Sequence from={at(24)}><Audio src={staticFile('audio/04_enter.mp3')} /></Sequence>
      {/* 0:27-0:30 安全操作 */}
      <Sequence from={at(27)}><Audio src={staticFile('audio/05_safe.mp3')} /></Sequence>
      {/* 0:31-0:36 中风险 */}
      <Sequence from={at(31)}><Audio src={staticFile('audio/06_medium.mp3')} /></Sequence>
      {/* 0:38-0:45 高风险 */}
      <Sequence from={at(38)}><Audio src={staticFile('audio/07_high.mp3')} /></Sequence>
      {/* 0:47-0:51 四个 tab */}
      <Sequence from={at(47)}><Audio src={staticFile('audio/08_tabs.mp3')} /></Sequence>
      {/* 0:54-1:01 CHAIN */}
      <Sequence from={at(54)}><Audio src={staticFile('audio/09_chain.mp3')} /></Sequence>
      {/* 1:07-1:13 CLI */}
      <Sequence from={at(67)}><Audio src={staticFile('audio/10_cli.mp3')} /></Sequence>
      {/* 1:18-1:23 开关联动 */}
      <Sequence from={at(78)}><Audio src={staticFile('audio/11_toggle.mp3')} /></Sequence>

      {/* === Subtitles === */}

      {/* 安装 */}
      <Subtitle zh="一行命令装好，自动注册 Hook" en="One command. Auto-registers Hook." startFrame={at(7)} duration={fps * 5} />
      <Subtitle zh="可选安装通知助手" en="Optional notification helper app." startFrame={at(12)} duration={fps * 3} />

      {/* 菜单栏 */}
      <Subtitle zh="统计、日志、规则、设置，都在菜单栏" en="Stats, logs, rules, settings — all in the menu bar." startFrame={at(16)} duration={fps * 6} />

      {/* 进入 Claude + 编码 */}
      <Subtitle zh="打开 Claude Code，正常给它任务" en="Open Claude Code. Give it a task." startFrame={at(24)} duration={fps * 3} />
      <Subtitle zh="安全操作直接放行，零延迟" en="Safe operations pass through. 0ms." startFrame={at(27)} duration={fps * 4} />

      {/* 中风险 */}
      <Subtitle zh="安装依赖，通知提醒，自动放行" en="Package install — notified, auto-allowed." startFrame={at(31)} duration={fps * 5} />

      {/* 高风险 */}
      <Subtitle zh="远程代码执行，立刻拦截" en="Remote code execution — blocked." startFrame={at(38)} duration={fps * 4} />
      <Subtitle zh="允许 / 拒绝 / 跳到终端" en="Allow / Deny / Skip to Terminal" startFrame={at(42)} duration={fps * 4} />

      {/* Tab 展示 */}
      <Subtitle zh="所有操作都有记录，四个面板随时查看" en="All operations logged. Four panels at a glance." startFrame={at(47)} duration={fps * 5} />

      {/* CHAIN + 终端 */}
      <Subtitle zh="操作链追踪，多步攻击也能识别" en="Chain tracking — multi-step attacks detected." startFrame={at(54)} duration={fps * 5} />
      <Subtitle zh="终端确认，你说了算" en="Terminal confirmation. You decide." startFrame={at(60)} duration={fps * 4} />

      {/* CLI 工具 */}
      <Subtitle zh="命令行工具也很全，状态、日志、缓存等" en="Full CLI toolkit: status, log, cache, and more." startFrame={at(67)} duration={fps * 7} />

      {/* 开关联动 */}
      <Subtitle zh="命令行和菜单栏完全同步" en="CLI and menu bar stay in sync." startFrame={at(78)} duration={fps * 8} />
    </div>
  );
};
