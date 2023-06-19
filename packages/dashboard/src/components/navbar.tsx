'use client';

import Link from 'next/link';

import { Button } from '~/components/ui';
import { useMeQuery } from '~/hooks';

export const Buttons: React.FC = () => {
  const { data } = useMeQuery();

  return data?.user ? (
    <Button asChild>
      <Link href="/dashboard">Dashboard</Link>
    </Button>
  ) : (
    <div className="space-x-3">
      <Button asChild variant="outline">
        <Link href="/login">Login</Link>
      </Button>
      <Button asChild>
        <Link href="/signup">Signup</Link>
      </Button>
    </div>
  );
};
