import {
  Avatar,
  AvatarFallback,
  AvatarImage,
  DashboardNavbar
} from '~/components';
import { cn } from '~/lib/utils';

const DashboardLayout = ({ children }: { children: React.ReactNode }) => (
  <section className="flex flex-row">
    <div className={cn('flex h-full w-screen justify-between border-b p-2')}>
      <DashboardNavbar />

      <Avatar>
        <AvatarImage src="https://github.com/xenfo.png" />
        <AvatarFallback>SC</AvatarFallback>
      </Avatar>
    </div>

    <div className="h-screen grow">{children}</div>
  </section>
);

export default DashboardLayout;
