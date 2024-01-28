import { createRootRoute, Outlet } from '@tanstack/react-router';
import { useAtomValue } from 'jotai';
import React, { Suspense, useEffect } from 'react';

import { useTokenRefreshQuery } from '~/hooks';
import { clientAtom, client as oldClient } from '~/lib';

export const Route = createRootRoute({
  component: RootComponent,
});

const TanStackRouterDevtools = import.meta.env.PROD
  ? () => null
  : React.lazy(() =>
      import('@tanstack/router-devtools').then((res) => ({
        default: res.TanStackRouterDevtools,
      })),
    );

function RootComponent() {
  const client = useAtomValue(clientAtom);

  const { data: accessToken } = useTokenRefreshQuery();

  useEffect(() => {
    if (accessToken) {
      oldClient.defaults.headers.common.Authorization = `Bearer ${accessToken}`;
      client.authToken = accessToken;
    }
  }, [accessToken, client]);

  return (
    <div className="bg-background text-primary flex h-screen flex-col font-['Work_Sans']">
      <Outlet />

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
