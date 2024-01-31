import {
  SiDiscord,
  SiFacebook,
  SiGithub,
  SiGoogle,
  SiTwitter,
} from '@icons-pack/react-simple-icons';
import { useSuspenseQueries } from '@tanstack/react-query';
import { createFileRoute, useRouterState } from '@tanstack/react-router';
import { Loader2 } from 'lucide-react';

import {
  Alert,
  AlertDescription,
  AlertTitle,
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '~/components';
import {
  meQueryOptions,
  tokenRefreshQueryOptions,
  useResendVerificationMutation,
} from '~/hooks';
import { providers } from '~/lib';

export const Route = createFileRoute('/dashboard/profile')({
  component: ProfileComponent,
});

const providerIcons = {
  google: <SiGoogle className="h-4 w-4" />,
  facebook: <SiFacebook className="h-4 w-4" />,
  github: <SiGithub className="h-4 w-4" />,
  twitter: <SiTwitter className="h-4 w-4" />,
  discord: <SiDiscord className="h-4 w-4" />,
};

function ProfileComponent() {
  const routerState = useRouterState();
  const client = Route.useRouteContext().client;

  const [{ data: accessToken }, { data: user }] = useSuspenseQueries({
    queries: [
      tokenRefreshQueryOptions(Route.useRouteContext().client),
      meQueryOptions(Route.useRouteContext()),
    ],
  });

  const { mutate, isPending } = useResendVerificationMutation();

  return (
    <div className="flex flex-col gap-y-5 p-5">
      {!user?.verified && (
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
            disabled={isPending}
          >
            {isPending && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            Resend verification
          </Button>
        </Alert>
      )}

      <Card>
        <CardHeader>
          <CardTitle>OAuth2</CardTitle>
          <CardDescription>
            Enable login with different providers for this account
          </CardDescription>
        </CardHeader>

        <CardContent>
          <div className="grid w-full grid-cols-5 gap-2">
            {providers.map((provider) => (
              <Button
                key={provider}
                asChild
                variant="outline"
                className="w-full"
              >
                <a
                  href={client.accounts.loginUsingOAuth2(provider, {
                    auth: accessToken,
                    to: routerState.location.pathname,
                  })}
                >
                  {providerIcons[provider]}
                </a>
              </Button>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
