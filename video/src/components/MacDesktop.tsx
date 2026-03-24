import React, { CSSProperties } from 'react';
import { AbsoluteFill, Img, staticFile } from 'remotion';

/**
 * Real macOS desktop background.
 * Crops out the Dock bar by scaling slightly and shifting up.
 */
interface MacDesktopProps {
  children: React.ReactNode;
  darken?: number;
  style?: CSSProperties;
}

export const MacDesktop: React.FC<MacDesktopProps> = ({
  children,
  darken = 0.3,
  style,
}) => {
  return (
    <AbsoluteFill style={style}>
      {/* Wallpaper — cropped to hide Dock and menubar widgets */}
      <Img
        src={staticFile('desktop.png')}
        style={{
          position: 'absolute',
          width: '110%',
          height: '115%',
          objectFit: 'cover',
          objectPosition: 'center 30%', // Shift up to crop Dock at bottom
          top: '-5%',
          left: '-5%',
        }}
      />
      {/* Darken overlay */}
      <div
        style={{
          position: 'absolute',
          inset: 0,
          background: `rgba(0, 0, 0, ${darken})`,
        }}
      />
      {/* Content */}
      <div style={{ position: 'relative', width: '100%', height: '100%', zIndex: 1 }}>
        {children}
      </div>
    </AbsoluteFill>
  );
};
