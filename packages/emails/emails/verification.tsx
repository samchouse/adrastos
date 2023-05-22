import {
  Body,
  Container,
  Head,
  Html,
  Preview,
  Tailwind
} from '@react-email/components';
import * as React from 'react';

interface VerificationProps {
  token: string;
}

export const VerificationEmail: React.FC<VerificationProps> = ({ token }) => (
  <Tailwind>
    <Html>
      <Head />
      <Preview>Verify your email for Adrastos</Preview>
      <Body>
        <Container></Container>
      </Body>
    </Html>
  </Tailwind>
);

export default VerificationEmail;
