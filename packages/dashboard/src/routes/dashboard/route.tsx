import { useSuspenseQueries } from '@tanstack/react-query';
import { createFileRoute, Link, Outlet } from '@tanstack/react-router';

import {
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  User,
} from '~/components';
import { meQueryOptions } from '~/hooks';
import { cn } from '~/lib/utils';

export const Route = createFileRoute('/dashboard')({
  component: RouteComponent,
});

function RouteComponent() {
  const { client } = Route.useRouteContext();

  const [{ data: user }] = useSuspenseQueries({
    queries: [meQueryOptions(Route.useRouteContext())],
  });

  return (
    <section className="flex h-full flex-col">
      <div
        className={cn(
          'bg-background absolute left-0 top-0 z-30 flex w-screen justify-between border-b px-4 py-3',
        )}
      >
        <div className="flex flex-row">
          <Link to="/dashboard">
            <img
              src="/logo.svg"
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
                  <Link to="/dashboard/tables">Tables</Link>
                </NavigationMenuLink>
              </NavigationMenuItem>
              <NavigationMenuItem>
                <NavigationMenuLink
                  asChild
                  className="focus:bg-accent focus:text-accent-foreground bg-background hover:bg-accent hover:text-accent-foreground data-[state=open]:bg-accent/50 data-[active]:bg-accent/50 group inline-flex h-10 w-max items-center justify-center rounded-md px-4 py-2 text-sm font-medium transition-colors focus:outline-none disabled:pointer-events-none disabled:opacity-50"
                >
                  <Link to="/dashboard/auth">Auth</Link>
                </NavigationMenuLink>
              </NavigationMenuItem>
            </NavigationMenuList>
          </NavigationMenu>
        </div>

        <User user={user} client={client} />
      </div>

      <div className="bg-background z-20 h-full">
        <Outlet />
      </div>
    </section>
  );
}
