'use client';

import { Alert, AlertDescription, AlertTitle, Button } from '~/components';
import { useMeQuery } from '~/hooks';

import { OAuth2Card } from './_components';

const Page: React.FC = () => {
  const { data } = useMeQuery();

  return (
    <div className="flex flex-col gap-y-5">
      {data?.user && (
        <Alert
          variant="default"
          className="flex flex-row items-center justify-between"
        >
          <div>
            <AlertTitle>Your email isn&apos;t verified</AlertTitle>
            <AlertDescription>
              Verify your email to unlock all features in the app
            </AlertDescription>
          </div>

          <Button variant="outline">Resend verification</Button>
        </Alert>
      )}

      <OAuth2Card />
    </div>
  );
};

export default Page;
