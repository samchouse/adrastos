import type { User as UserType } from '@adrastos/lib';
import { SiGithub } from '@icons-pack/react-simple-icons';
import { Link } from '@tanstack/react-router';

import type { Project, Team } from '~/types';

import { Button, TeamCombobox, User } from '.';

interface PropsWithUserBreadcrumb {
  user: UserType;
  breadcrumbUser: boolean;
}

interface PropsWithTeams {
  teams: Team[];
  user: UserType;
  teamId: string;
}

interface PropsWithProject extends PropsWithTeams {
  project: Project;
}

export const Navbar: React.FC<
  | {
      user?: UserType;
    }
  | PropsWithUserBreadcrumb
  | PropsWithTeams
  | PropsWithProject
> = ({ user, ...props }) => (
  <header>
    <nav className="flex w-screen select-none flex-col justify-between space-y-3 border-b bg-background px-4 pt-3 pb-2">
      <div className="flex w-full flex-row justify-between">
        <div className="flex flex-row items-center">
          <Link
            className="flex flex-row items-center"
            {...('teamId' in props
              ? {
                  to: '/dashboard/teams/$teamId',
                  params: { teamId: props.teamId },
                }
              : {
                  to: 'breadcrumbUser' in props ? '/dashboard' : '/',
                  params: {},
                })}
          >
            <img
              alt="logo"
              width={40}
              height={40}
              src="/logo.svg"
              className="mr-2"
            />
            <h1 className="ml-2 font-semibold text-xl">Adrastos</h1>
          </Link>
          {('teams' in props || 'breadcrumbUser' in props) && (
            <>
              <p className="mx-4 font-medium text-3xl text-muted">/</p>
              {'teams' in props ? (
                <>
                  <TeamCombobox teams={props.teams} teamId={props.teamId} />
                  {'project' in props && (
                    <>
                      <p className="mr-4 ml-2 font-medium text-3xl text-muted">
                        /
                      </p>
                      <Link
                        className="font-medium"
                        to="/dashboard/projects/$projectId"
                        params={{ projectId: props.project.id }}
                      >
                        {props.project.name}
                      </Link>
                    </>
                  )}
                </>
              ) : (
                <Link to="/dashboard/profile" className="font-medium">
                  {user?.firstName} {user?.lastName}
                </Link>
              )}
            </>
          )}
        </div>

        {('teamId' in props || 'breadcrumbUser' in props) && user ? (
          <div className="flex flex-row items-center">
            <div className="mr-4 flex flex-row items-center">
              <Button variant="ghost">Changelog</Button>
              <Button variant="ghost">Docs</Button>
              <Button asChild size="icon" variant="ghost">
                <a
                  target="_blank"
                  rel="noreferrer noopener"
                  href="https://github.com/samchouse/adrastos"
                >
                  <SiGithub className="size-4" />
                </a>
              </Button>
            </div>

            <User user={user} />
          </div>
        ) : user ? (
          <Button asChild>
            <Link to="/dashboard">Dashboard</Link>
          </Button>
        ) : (
          <div className="space-x-3">
            <Button asChild variant="outline">
              <Link to="/login">Login</Link>
            </Button>
            <Button asChild>
              <Link to="/register">Register</Link>
            </Button>
          </div>
        )}
      </div>

      {'project' in props ? (
        <div className="space-x-1">
          <Button asChild size="sm" variant="ghost">
            <Link
              activeOptions={{ exact: true }}
              to="/dashboard/projects/$projectId"
              params={{ projectId: props.project.id }}
              className="text-muted-foreground hover:bg-accent/70 data-[status=active]:bg-accent data-[status=active]:text-primary"
            >
              Overview
            </Link>
          </Button>
          <Button asChild size="sm" variant="ghost">
            <Link
              to="/dashboard/projects/$projectId/auth"
              params={{ projectId: props.project.id }}
              className="text-muted-foreground hover:bg-accent/70 data-[status=active]:bg-accent data-[status=active]:text-primary"
            >
              Auth
            </Link>
          </Button>
          <Button asChild size="sm" variant="ghost">
            <Link
              params={{ projectId: props.project.id }}
              to="/dashboard/projects/$projectId/tables"
              className="text-muted-foreground hover:bg-accent/70 data-[status=active]:bg-accent data-[status=active]:text-primary"
            >
              Tables
            </Link>
          </Button>
          <Button asChild size="sm" variant="ghost">
            <Link
              params={{ projectId: props.project.id }}
              to="/dashboard/projects/$projectId/storage"
              className="text-muted-foreground hover:bg-accent/70 data-[status=active]:bg-accent data-[status=active]:text-primary"
            >
              Storage
            </Link>
          </Button>
          <Button asChild size="sm" variant="ghost">
            <Link
              params={{ projectId: props.project.id }}
              to="/dashboard/projects/$projectId/settings"
              className="text-muted-foreground hover:bg-accent/70 data-[status=active]:bg-accent data-[status=active]:text-primary"
            >
              Settings
            </Link>
          </Button>
        </div>
      ) : (
        'teamId' in props && (
          <div className="space-x-1">
            <Button asChild size="sm" variant="ghost">
              <Link
                to="/dashboard/teams/$teamId"
                activeOptions={{ exact: true }}
                params={{ teamId: props.teamId }}
                className="text-muted-foreground hover:bg-accent/70 data-[status=active]:bg-accent data-[status=active]:text-primary"
              >
                Projects
              </Link>
            </Button>
            <Button asChild size="sm" variant="ghost">
              <Link
                activeOptions={{ exact: true }}
                params={{ teamId: props.teamId }}
                to="/dashboard/teams/$teamId/settings"
                className="text-muted-foreground hover:bg-accent/70 data-[status=active]:bg-accent data-[status=active]:text-primary"
              >
                Settings
              </Link>
            </Button>
          </div>
        )
      )}
    </nav>
  </header>
);
