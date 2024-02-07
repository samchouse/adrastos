import { Client, User as UserType } from '@adrastos/lib';
import { useQueryClient } from '@tanstack/react-query';
import {
  Link,
  useNavigate,
  useRouter,
  useRouterState,
} from '@tanstack/react-router';
import { ExternalLink, LogOut, Settings, User as UserIcon } from 'lucide-react';

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
import { queryKeys, useLogoutMutation } from '~/hooks';

export const User: React.FC<{
  client: Client;
  user: UserType;
}> = ({ user, client }) => {
  const navigate = useNavigate();
  const router = useRouter();
  const routerState = useRouterState();

  const queryClient = useQueryClient();

  const { mutateAsync } = useLogoutMutation(client);

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
            <p className="text-muted-foreground text-xs leading-none">
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
        <Link to="/dashboard/settings">
          <DropdownMenuItem className="cursor-pointer">
            <Settings className="mr-2 h-4 w-4" />
            <span>Settings</span>
          </DropdownMenuItem>
        </Link>
        <DropdownMenuSeparator />
        <Link to="/home">
          <DropdownMenuItem className="cursor-pointer">
            <ExternalLink className="mr-2 h-4 w-4" />
            <span>Home</span>
          </DropdownMenuItem>
        </Link>
        <DropdownMenuSeparator />
        <DropdownMenuItem
          onSelect={() =>
            void (async () => {
              await mutateAsync();
              await queryClient.setQueryData(queryKeys.tokenRefresh, null);
              await navigate({
                to: '/login',
                search: { to: routerState.location.pathname },
              });
              router.invalidate();
              await queryClient.resetQueries({
                queryKey: queryKeys.tokenRefresh,
              });
              await queryClient.resetQueries({ queryKey: queryKeys.me });
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
