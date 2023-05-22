import {
  Body,
  Button,
  Container,
  Head,
  Heading,
  Hr,
  Html,
  Preview,
  Tailwind,
  Text
} from '@react-email/components';
import * as React from 'react';

interface VerificationProps {
  token: string;
}

const Font = ({
  webFont,
  fontStyle = 'normal',
  fontFamily,
  fontWeight = 400,
  fallbackFontFamily
}) => {
  const src = webFont
    ? `src: url(${webFont.url}) format(${webFont.format});`
    : '';

  return (
    <style>
      {`
          @font-face {
              font-style: ${fontStyle};
              font-family: 'Work Sans';
              font-weight: ${fontWeight};
              mso-font-alt: ${
                Array.isArray(fallbackFontFamily)
                  ? fallbackFontFamily[0]
                  : fallbackFontFamily
              };
              ${src}
          }

          * {
              font-family: ${fontFamily}, ${
        Array.isArray(fallbackFontFamily)
          ? fallbackFontFamily.join(', ')
          : fallbackFontFamily
      };
          }
          `}
    </style>
  );
};

export const VerificationEmail: React.FC<VerificationProps> = ({
  token = 'test'
}) => (
  <Tailwind>
    <Html lang="en" className="bg-white">
      <Head>
        <Font
          fontFamily="Roboto"
          fallbackFontFamily="Helvetica"
          webFont={{
            url: 'https://fonts.gstatic.com/s/roboto/v27/KFOmCnqEu92Fr1Mu4mxKKTU1Kg.woff2',
            format: 'woff2'
          }}
        />
      </Head>

      <Preview>Verify your email for Adrastos</Preview>

      <Body>
        <Container>
          <Heading as="h2">Verify your email</Heading>
          <Hr />

          <Text>
            Click the button bellow to verify your account for Adrastos.
          </Text>
          <Button href={`https://localhost:8000/auth/verify?token=${token}`}>
            Click to verify
          </Button>
        </Container>
      </Body>
    </Html>
  </Tailwind>
);

export default VerificationEmail;
