import { User as UserType } from '@adrastos/lib';
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
        <Button
          variant="ghost"
          className="relative h-[40px] w-[40px] rounded-full"
        >
          <Avatar className="h-[40px] w-[40px]">
            <AvatarImage
              src={`https://github.com/${user.username}.png`}
              alt={`@${user.username}`}
            />
            <AvatarFallback>{`${user.firstName[0]}${user.lastName[0]}`}</AvatarFallback>
          </Avatar>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-56">
        <DropdownMenuLabel className="font-normal">
          <div className="flex flex-col space-y-1">
            <p className="text-sm font-medium leading-none">{user.username}</p>
            <p className="text-xs leading-none text-muted-foreground">
              {user.email}
            </p>
          </div>
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <Link to="/dashboard/profile">
          <DropdownMenuItem className="cursor-pointer">
            <UserIcon className="mr-2 h-4 w-4" />
            <span>Profile</span>
          </DropdownMenuItem>
        </Link>
        <DropdownMenuSeparator />
        <DropdownMenuItem
          onSelect={() =>
            void (async () => {
              await mutateAsync();
              await navigate({
                to: '/login',
                search: { to: routerState.location.pathname },
              });
            })()
          }
          className="cursor-pointer"
        >
          <LogOut className="mr-2 h-4 w-4" />
          <span>Logout</span>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};
