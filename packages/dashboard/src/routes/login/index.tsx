import { createFileRoute } from '@tanstack/react-router';
import { z } from 'zod';

export const Route = createFileRoute('/login/')({
  validateSearch: (search) =>
    z
      .object({
        to: z.string().optional(),
      })
      .parse(search),
});
