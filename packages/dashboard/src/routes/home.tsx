import { createFileRoute } from '@tanstack/react-router';

import { IndexComponent } from '.';

export const Route = createFileRoute('/home')({
  component: IndexComponent,
});
