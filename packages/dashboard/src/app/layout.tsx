import './globals.css';

import { StaticImport } from 'next/dist/shared/lib/get-img-props';
import { Work_Sans as WorkSans } from 'next/font/google';
import Image from 'next/image';
import Link from 'next/link';

import { Auth, Buttons, Providers } from '~/components';
import { cn } from '~/lib/utils';

import logo from '../../public/logo.svg';

const workSans = WorkSans({ subsets: ['latin'] });

export const metadata = {
  title: 'Adrastos',
  description: 'A killer Backend-as-a-Service (BaaS) written in Rust',
};

const RootLayout: React.FC<React.PropsWithChildren> = ({ children }) => (
  <html lang="en" className="dark">
    <body
      className={cn(
        workSans.className,
        'bg-background text-primary flex h-screen flex-col',
      )}
    >
      <Providers>
        <header
          className={cn(
            'bg-background relative z-10 flex w-screen justify-between border-b px-4 py-3',
          )}
        >
          <Link href="/">
            <Image
              src={logo as StaticImport}
              alt="logo"
              width={40}
              height={40}
              className="mr-2"
            />
          </Link>

          <Buttons />
        </header>

        <main className="bg-background h-full">
          <Auth>{children}</Auth>
        </main>
      </Providers>
    </body>
  </html>
);

export default RootLayout;
