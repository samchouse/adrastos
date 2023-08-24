'use client';

import Link from 'next/link';
import { title } from 'radash';

import { Button } from '~/components';
import { useTablesQuery } from '~/hooks';

import { CreateSheet } from './_components';

const TablesLayout: React.FC<React.PropsWithChildren> = ({ children }) => {
  const { data } = useTablesQuery();

  return (
    <section className="flex h-full w-full flex-row">
      <div className="flex h-full w-72 flex-col border-r">
        <div className="m-4">
          <CreateSheet />

          <h2 className="mb-2 text-lg font-semibold">Tables</h2>
          {data?.tables.map((table) => (
            <Link key={table.id} href={`/dashboard/tables/${table.name}`}>
              <Button variant="ghost" className="w-full justify-start">
                {title(table.name)}
              </Button>
            </Link>
          ))}
        </div>
      </div>

      <div className="w-full">{children}</div>
    </section>
  );
};

export default TablesLayout;
