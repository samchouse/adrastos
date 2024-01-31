import { useSuspenseQuery } from '@tanstack/react-query';
import {
  createRootRouteWithContext,
  Link,
  Outlet,
  redirect,
} from '@tanstack/react-router';
import React, { Suspense, useEffect } from 'react';
import Cookies from 'universal-cookie';

import { Button } from '~/components/ui';
import { tokenRefreshQueryOptions, useMeQuery } from '~/hooks';
import { client as oldClient } from '~/lib';
import { RouterContext } from '~/typings';

export const Route = createRootRouteWithContext<RouterContext>()({
  component: RootComponent,
  loader: async ({ context: { client, queryClient } }) => ({
    accessToken: await queryClient
      .ensureQueryData(tokenRefreshQueryOptions(client))
      .catch(() => undefined),
  }),
  beforeLoad: async ({ location, context: { client, queryClient } }) => {
    const cookies = new Cookies();
    const isLoggedIn = cookies.get('isLoggedIn') as boolean | undefined;

    if (['/', '/login', '/signup'].includes(location.pathname) && isLoggedIn)
      throw redirect({
        to: '/dashboard',
      });

    if (location.pathname.startsWith('/dashboard') && !isLoggedIn)
      throw redirect({
        to: '/login',
        search: {
          to: location.pathname,
        },
      });

    client.authToken = await queryClient
      .ensureQueryData(tokenRefreshQueryOptions(client))
      .catch(() => undefined);
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
  const { data: user } = useMeQuery();
  const { data: accessToken } = useSuspenseQuery(
    tokenRefreshQueryOptions(Route.useRouteContext().client),
  );

  useEffect(() => {
    if (accessToken)
      oldClient.defaults.headers.common.Authorization = `Bearer ${accessToken}`;
  }, [accessToken]);

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
