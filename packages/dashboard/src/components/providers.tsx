'use client';

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';
import { Provider } from 'jotai';
import { DevTools as JotaiDevtools } from 'jotai-devtools';
import { useState } from 'react';

import { Toaster } from '~/components/ui';

export const Providers: React.FC<React.PropsWithChildren> = ({ children }) => {
  const [queryClient] = useState(() => new QueryClient());

  return (
    <Provider>
      <QueryClientProvider client={queryClient}>
        <Toaster />
        <JotaiDevtools theme="dark" />
        <ReactQueryDevtools position="bottom" />

        {children}
      </QueryClientProvider>
    </Provider>
  );
};
