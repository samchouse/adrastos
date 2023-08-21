'use client';

import { Plus } from 'lucide-react';
import Link from 'next/link';
import { title } from 'radash';

import { Button } from '~/components';
import { useTablesQuery } from '~/hooks';

const TablesLayout: React.FC<React.PropsWithChildren> = ({ children }) => {
  const { data } = useTablesQuery();

  return (
    <section className="flex h-full w-full flex-row">
      <div className="mr-5 flex h-full w-[240px] flex-col border-r-2 pr-5">
        <Button className="mb-5 w-full">
          <Plus className="mr-2 h-4 w-4" /> Create New
        </Button>

        <h2 className="mb-2 ml-3 text-lg font-semibold">Tables</h2>
        {data?.tables.map((table) => (
          <Link key={table.id} href={`/dashboard/tables/${table.name}`}>
            <Button variant="ghost" className="w-full justify-start">
              {title(table.name)}
            </Button>
          </Link>
        ))}
      </div>

      <div className="w-full">{children}</div>
    </section>
  );
};

export default TablesLayout;
