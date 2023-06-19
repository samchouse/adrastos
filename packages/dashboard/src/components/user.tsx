'use client';

import { ExternalLink, LogOut, Settings, User as UserIcon } from 'lucide-react';
import Link from 'next/link';

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
  Skeleton
} from '~/components/ui';
import { getLogout } from '~/lib';
import { User as UserType } from '~/types';

export const User: React.FC<{ user?: UserType }> = ({ user }) =>
  user ? (
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
        <Link href="/dashboard/profile">
          <DropdownMenuItem className="cursor-pointer">
            <UserIcon className="mr-2 h-4 w-4" />
            <span>Profile</span>
          </DropdownMenuItem>
        </Link>
        <Link href="/dashboard/settings">
          <DropdownMenuItem className="cursor-pointer">
            <Settings className="mr-2 h-4 w-4" />
            <span>Settings</span>
          </DropdownMenuItem>
        </Link>
        <DropdownMenuSeparator />
        <Link href="/home">
          <DropdownMenuItem className="cursor-pointer">
            <ExternalLink className="mr-2 h-4 w-4" />
            <span>Home</span>
          </DropdownMenuItem>
        </Link>
        <DropdownMenuSeparator />
        <DropdownMenuItem onSelect={getLogout} className="cursor-pointer">
          <LogOut className="mr-2 h-4 w-4" />
          <span>Logout</span>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  ) : (
    <Skeleton className="h-[40px] w-[40px] rounded-full" />
  );
