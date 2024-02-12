import {
  Body,
  Button,
  Container,
  Head,
  Heading,
  Hr,
  Html,
  Link,
  Preview,
  Tailwind,
  Text,
} from '@react-email/components';
import * as React from 'react';

import { Font } from '../components';

const supportEmail = 'support@adrastos.xenfo.dev';
const verificationUrl = (baseUrl: string, token: string) =>
  `${baseUrl}/auth/verify?token=${token}`;

interface VerificationProps {
  token: string;
  baseUrl: string;
}

export const VerificationEmail: React.FC<VerificationProps> = ({
  token,
  baseUrl,
}) => {
  const url = verificationUrl(baseUrl, token);

  return (
    <Tailwind>
      <Html className="bg-white">
        <Head>
          <Font
            format="woff2"
            fallbackFontFamily={['Helvetica', 'Verdana', 'Georgia']}
            fonts={[
              {
                family: 'Roboto',
                weight: 400,
                url: 'https://fonts.gstatic.com/s/roboto/v30/KFOmCnqEu92Fr1Mu4mxK.woff2',
              },
              {
                family: 'Roboto-Bold',
                weight: 700,
                url: 'https://fonts.gstatic.com/s/roboto/v30/KFOlCnqEu92Fr1MmWUlfBBc4.woff2',
              },
            ]}
          />
        </Head>

        <Preview>Verify your email for Adrastos</Preview>

        <Body>
          <Container className="rounded-xl border-2 border-solid border-gray-200 bg-gray-50 px-5 pb-2 text-center">
            <Heading
              as="h2"
              className="text-2xl font-bold"
              style={{
                fontFamily: 'Roboto-Bold, Roboto',
              }}
            >
              Verify Your Email
            </Heading>
            <Hr className="border-gray-200" />

            <Text className="text-base">
              Your email was used to sign up for Adrastos. Click the button
              bellow to verify your account.
            </Text>

            <Button
              href={url}
              className="rounded-lg bg-gray-900 px-4 py-3 text-base text-gray-50"
            >
              Click To Verify
            </Button>

            <Text className="text-sm">
              Or if the button doesn&apos;t work, copy and paste the link bellow
              into your browser.
              <br />
              <Link href={url}>{url}</Link>
            </Text>
            <Hr className="border-gray-200" />

            <Text className="text-sm">
              If you did not perform this action yourself, please contact{' '}
              <Link href={`mailto:${supportEmail}`}>{supportEmail}</Link> to
              report it.
            </Text>
          </Container>
        </Body>
      </Html>
    </Tailwind>
  );
};

export default VerificationEmail;
