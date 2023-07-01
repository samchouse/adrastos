'use client';

import { Loader2 } from 'lucide-react';

import { Alert, AlertDescription, AlertTitle, Button } from '~/components';
import { useMeQuery, useResendVerificationMutation } from '~/hooks';

import { OAuth2Card } from './_components';

const Page: React.FC = () => {
  const { data } = useMeQuery();
  const { mutate, isLoading } = useResendVerificationMutation();

  return (
    <div className="flex flex-col gap-y-5">
      {!data?.user.verified && (
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

          <Button
            variant="outline"
            onClick={() => mutate()}
            disabled={isLoading}
          >
            {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            Resend verification
          </Button>
        </Alert>
      )}

      <OAuth2Card />
    </div>
  );
};

export default Page;
