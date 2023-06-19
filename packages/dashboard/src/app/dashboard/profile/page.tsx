'use client';

import { Alert, AlertDescription, AlertTitle, Button } from '~/components';
import { useMeQuery } from '~/hooks';

const Page: React.FC = () => {
  const { data } = useMeQuery();

  return (
    <>
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
    </>
  );
};

export default Page;
