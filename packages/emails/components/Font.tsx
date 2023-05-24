import { FontProps as EmailFontProps } from '@react-email/components';
import * as React from 'react';

type FontProps = Pick<EmailFontProps, 'fallbackFontFamily'> & {
  fonts: { url: string; family: string; weight: number }[];
  format: NonNullable<EmailFontProps['webFont']>['format'];
};

export const Font: React.FC<FontProps> = ({
  fonts,
  format,
  fallbackFontFamily
}) => {
  const fontFaceStyles = fonts.map(
    (font) => `@font-face {
        font-family: ${font.family};
        font-style: normal;
        font-weight: ${font.weight};
        font-display: swap;
        mso-font-alt: ${
          Array.isArray(fallbackFontFamily)
            ? fallbackFontFamily[0]
            : fallbackFontFamily
        };
        src: url(${font.url}) format(${format});
        unicode-range: U+0000-00FF, U+0131, U+0152-0153, U+02BB-02BC, U+02C6, U+02DA, U+02DC, U+0304, U+0308, U+0329, U+2000-206F, U+2074, U+20AC, U+2122, U+2191, U+2193, U+2212, U+2215, U+FEFF, U+FFFD;
      }`
  );

  return (
    <style>
      {`
          ${fontFaceStyles.join('\n')}

          * {
              font-family: ${fonts.map((font) => font.family)}, ${
        Array.isArray(fallbackFontFamily)
          ? fallbackFontFamily.join(', ')
          : fallbackFontFamily
      }, sans-serif;
          }
      `}
    </style>
  );
};
