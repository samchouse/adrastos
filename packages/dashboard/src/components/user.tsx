import type { User as UserType } from '@adrastos/lib';
import { Link, useNavigate, useRouterState } from '@tanstack/react-router';
import { LogOut, User as UserIcon } from 'lucide-react';

import {
  Avatar,
  AvatarFallback,
  AvatarImage,
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '~/components/ui';
import { useLogoutMutation } from '~/hooks';

export const User: React.FC<{
  user: UserType;
}> = ({ user }) => {
  const navigate = useNavigate();
  const routerState = useRouterState();

  const { mutateAsync } = useLogoutMutation();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" className="relative size-[40px] rounded-full">
          <Avatar className="size-[40px]">
            <AvatarImage
              alt={`@${user.username}`}
              src={`https://github.com/${user.username}.png`}
            />
            <AvatarFallback>{`${user.firstName[0]}${user.lastName[0]}`}</AvatarFallback>
          </Avatar>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-56">
        <DropdownMenuLabel className="font-normal">
          <div className="flex flex-col space-y-1">
            <p className="font-medium text-sm leading-none">{user.username}</p>
            <p className="text-muted-foreground text-xs leading-none">
              {user.email}
            </p>
          </div>
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <Link to="/dashboard/profile">
          <DropdownMenuItem className="cursor-pointer">
            <UserIcon className="mr-2 size-4" />
            <span>Profile</span>
          </DropdownMenuItem>
        </Link>
        <DropdownMenuSeparator />
        <DropdownMenuItem
          className="cursor-pointer"
          onSelect={() =>
            void (async () => {
              await mutateAsync();
              await navigate({
                to: '/login',
                search: { to: routerState.location.pathname },
              });
            })()
          }
        >
          <LogOut className="mr-2 size-4" />
          <span>Logout</span>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};
