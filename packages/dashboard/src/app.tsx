import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';
import { createRouter, RouterProvider } from '@tanstack/react-router';
import { Provider } from 'jotai';
import { DevTools as JotaiDevtools } from 'jotai-devtools';
import { StrictMode } from 'react';
import ReactDOM from 'react-dom/client';

import { Toaster } from '~/components/ui';

import { routeTree } from './routeTree.gen';

import './index.css';

const queryClient = new QueryClient();
const router = createRouter({ routeTree });

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <Provider>
      <QueryClientProvider client={queryClient}>
        <Toaster />
        <JotaiDevtools theme="dark" />
        <ReactQueryDevtools position="bottom" />

        <RouterProvider router={router} />
      </QueryClientProvider>
    </Provider>
  </StrictMode>,
);
