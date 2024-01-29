import { createLazyFileRoute } from '@tanstack/react-router';

import { IndexComponent } from './index.lazy';

export const Route = createLazyFileRoute('/home')({
  component: IndexComponent,
});
