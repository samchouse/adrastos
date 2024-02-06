import { User } from '@adrastos/lib';
import {
  createRootRouteWithContext,
  Link,
  Outlet,
  redirect,
} from '@tanstack/react-router';
import { useAtomValue } from 'jotai';
import React, { Suspense, useEffect } from 'react';

import { Button } from '~/components/ui';
import { meQueryOptions, tokenRefreshQueryOptions } from '~/hooks';
import { clientAtom } from '~/lib';
import { RouterContext } from '~/typings';

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootComponent,
  loader: async ({ context: { client, queryClient } }) => ({
    accessToken: await queryClient
      .ensureQueryData(tokenRefreshQueryOptions(client))
      .catch(() => undefined),
    user: await queryClient
      .ensureQueryData(meQueryOptions(client))
      .catch(() => undefined),
  }),
  beforeLoad: async ({ location, context: { client, queryClient } }) => {
    client.authToken = await queryClient
      .ensureQueryData(tokenRefreshQueryOptions(client))
      .catch(() => undefined);

    if (
      ['/', '/login', '/signup'].includes(location.pathname) &&
      client.hasAuthToken
    )
      throw redirect({
        to: '/dashboard',
      });

    if (location.pathname.startsWith('/dashboard') && !client.hasAuthToken)
      throw redirect({
        to: '/login',
        search: {
          to: location.pathname,
        },
      });
  },
});

const TanStackRouterDevtools = import.meta.env.PROD
  ? () => null
  : React.lazy(() =>
      import('@tanstack/router-devtools').then((res) => ({
        default: res.TanStackRouterDevtools,
      })),
    );

function RootComponent() {
  const anotherOldClient = useAtomValue(clientAtom);
  const { accessToken, user } = Route.useLoaderData<{
    accessToken: string | undefined;
    user: User | undefined;
  }>();

  useEffect(() => {
    if (accessToken) {
      anotherOldClient.authToken = accessToken;
    }
  }, [accessToken, anotherOldClient]);

  return (
    <div className="bg-background text-primary flex h-screen flex-col font-['Work_Sans']">
      <header className="bg-background relative z-10 flex w-screen justify-between border-b px-4 py-3">
        <Link to="/">
          <img
            src="/logo.svg"
            alt="logo"
            width={40}
            height={40}
            className="mr-2"
          />
        </Link>

        {user ? (
          <Button asChild>
            <Link to="/dashboard">Dashboard</Link>
          </Button>
        ) : (
          <div className="space-x-3">
            <Button asChild variant="outline">
              <Link to="/login">Login</Link>
            </Button>
            <Button asChild>
              <Link to="/signup">Signup</Link>
            </Button>
          </div>
        )}
      </header>

      <main className="bg-background h-full overflow-y-auto">
        <Outlet />
      </main>

      <Suspense fallback={null}>
        <TanStackRouterDevtools
          position="bottom-right"
          toggleButtonProps={{ style: { bottom: '70px' } }}
          closeButtonProps={{ style: { bottom: '70px' } }}
        />
      </Suspense>
    </div>
  );
}
