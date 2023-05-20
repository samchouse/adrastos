import {
  Avatar,
  AvatarFallback,
  AvatarImage,
  DashboardNavbar,
  DashboardNavbar2
} from '~/components';
import { cn } from '~/lib/utils';

const DashboardLayout = ({ children }: { children: React.ReactNode }) => (
  <section className="flex flex-row">
    {/* <div className="flex w-[275px] flex-col justify-between border-r"> */}
    {/* <DashboardNavbar /> */}
    <div className={cn('flex h-full w-screen justify-between border-b p-2')}>
      <DashboardNavbar2 />

      <Avatar>
        <AvatarImage src="https://github.com/xenfo.png" />
        <AvatarFallback>SC</AvatarFallback>
      </Avatar>
    </div>
    {/* 
      <div className="p-3">
        <Avatar className="h-12 w-12">
          <AvatarImage src="https://github.com/xenfo.png" />
          <AvatarFallback>SC</AvatarFallback>
        </Avatar>
      </div>
    </div> */}

    <div className="h-screen grow">{children}</div>
  </section>
);

export default DashboardLayout;
