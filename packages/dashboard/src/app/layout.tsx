import './globals.css';

import { Work_Sans as WorkSans } from 'next/font/google';
import Link from 'next/link';

import {
  Button,
  NavigationMenu,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList
} from '~/components';
import { cn } from '~/lib/utils';

const workSans = WorkSans({ subsets: ['latin'] });

export const metadata = {
  title: 'Adrastos',
  description: 'A killer Backend-as-a-Service (BaaS) written in Rust'
};

const RootLayout = ({ children }: { children: React.ReactNode }) => (
  <html lang="en" className="dark">
    <body className={cn(workSans.className, 'bg-background text-primary')}>
      <div
        className={cn(
          'bg-background absolute left-0 top-0 z-50 flex w-screen justify-between border-b p-2'
        )}
      >
        <NavigationMenu className="flex-none">
          <NavigationMenuList>
            <NavigationMenuItem>
              <NavigationMenuLink
                asChild
                className="focus:bg-accent focus:text-accent-foreground bg-background hover:bg-accent hover:text-accent-foreground data-[state=open]:bg-accent/50 data-[active]:bg-accent/50 group inline-flex h-10 w-max items-center justify-center rounded-md px-4 py-2 text-sm font-medium transition-colors focus:outline-none disabled:pointer-events-none disabled:opacity-50"
              >
                <Link href="/">Home</Link>
              </NavigationMenuLink>
              <NavigationMenuLink
                asChild
                className="focus:bg-accent focus:text-accent-foreground bg-background hover:bg-accent hover:text-accent-foreground data-[state=open]:bg-accent/50 data-[active]:bg-accent/50 group inline-flex h-10 w-max items-center justify-center rounded-md px-4 py-2 text-sm font-medium transition-colors focus:outline-none disabled:pointer-events-none disabled:opacity-50"
              >
                <Link href="/dashboard">Dashboard</Link>
              </NavigationMenuLink>
            </NavigationMenuItem>
          </NavigationMenuList>
        </NavigationMenu>

        <div className="mr-4 space-x-3">
          <Button asChild variant="outline">
            <Link href="/login">Login</Link>
          </Button>
          <Button asChild className="">
            <Link href="/signup">Signup</Link>
          </Button>
        </div>
      </div>

      {children}
    </body>
  </html>
);

export default RootLayout;
