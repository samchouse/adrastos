'use client';

import Image from 'next/image';
import Link from 'next/link';
import { usePathname, useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';

import {
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  User,
} from '~/components';
import { useMeQuery, useTokenRefreshQuery } from '~/hooks';
import { cn } from '~/lib/utils';

import logo from '../../../public/logo.svg';

const DashboardLayout: React.FC<React.PropsWithChildren> = ({ children }) => {
  const router = useRouter();
  const pathname = usePathname();
  const [isLoggingOff, setIsLoggingOff] = useState(false);

  const { data } = useMeQuery();
  const { isError } = useTokenRefreshQuery();

  useEffect(() => {
    if (isError && !isLoggingOff && pathname.includes('/dashboard'))
      router.push(`/login?to=${pathname}`);
  }, [data, isError, isLoggingOff, pathname, router]);

  return (
    <section className="flex h-full flex-col">
      <div
        className={cn(
          'bg-background absolute left-0 top-0 z-30 flex w-screen justify-between border-b px-4 py-3',
        )}
      >
        <div className="flex flex-row">
          <Link href="/dashboard">
            <Image
              src={logo}
              alt="logo"
              width={40}
              height={40}
              className="mr-2"
            />
          </Link>

          <NavigationMenu className="flex-none">
            <NavigationMenuList>
              <NavigationMenuItem>
                <NavigationMenuLink
                  asChild
                  className="focus:bg-accent focus:text-accent-foreground bg-background hover:bg-accent hover:text-accent-foreground data-[state=open]:bg-accent/50 data-[active]:bg-accent/50 group inline-flex h-10 w-max items-center justify-center rounded-md px-4 py-2 text-sm font-medium transition-colors focus:outline-none disabled:pointer-events-none disabled:opacity-50"
                >
                  <Link href="/dashboard/tables">Tables</Link>
                </NavigationMenuLink>
              </NavigationMenuItem>
              <NavigationMenuItem>
                <NavigationMenuLink
                  asChild
                  className="focus:bg-accent focus:text-accent-foreground bg-background hover:bg-accent hover:text-accent-foreground data-[state=open]:bg-accent/50 data-[active]:bg-accent/50 group inline-flex h-10 w-max items-center justify-center rounded-md px-4 py-2 text-sm font-medium transition-colors focus:outline-none disabled:pointer-events-none disabled:opacity-50"
                >
                  <Link href="/dashboard/auth">Auth</Link>
                </NavigationMenuLink>
              </NavigationMenuItem>
            </NavigationMenuList>
          </NavigationMenu>
        </div>

        <User user={data?.user} setIsLoggingOff={setIsLoggingOff} />
      </div>

      <div className="bg-background z-20 h-full">{children}</div>
    </section>
  );
};

export default DashboardLayout;
