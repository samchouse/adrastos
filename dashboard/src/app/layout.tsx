import './globals.css';

import clsx from 'clsx';
import { Work_Sans as WorkSans } from 'next/font/google';

const workSans = WorkSans({ subsets: ['latin'] });

export const metadata = {
  title: 'Adrastos',
  description: 'A killer Backend-as-a-Service (BaaS) written in Rust'
};

const RootLayout = ({ children }: { children: React.ReactNode }) => (
  <html lang="en" className="dark">
    <body className={clsx(workSans.className, 'bg-background text-primary')}>
      {children}
    </body>
  </html>
);

export default RootLayout;
