import { User as UserType } from '@adrastos/lib';
import { SiGithub } from '@icons-pack/react-simple-icons';
import { Link } from '@tanstack/react-router';

import { Project, Team } from '~/types';

import { Button, TeamCombobox, User } from '.';

interface PropsWithUserBreadcrumb {
  user: UserType;
  breadcrumbUser: boolean;
}

interface PropsWithTeams {
  user: UserType;
  teamId: string;
  teams: Team[];
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
    <nav className="flex w-screen select-none flex-col justify-between space-y-3 border-b bg-background px-4 pb-2 pt-3">
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
              src="/logo.svg"
              alt="logo"
              width={40}
              height={40}
              className="mr-2"
            />
            <h1 className="ml-2 text-xl font-semibold">Adrastos</h1>
          </Link>
          {('teams' in props || 'breadcrumbUser' in props) && (
            <>
              <p className="mx-4 text-3xl font-medium text-muted">/</p>
              {'teams' in props ? (
                <>
                  <TeamCombobox teamId={props.teamId} teams={props.teams} />
                  {'project' in props && (
                    <>
                      <p className="ml-2 mr-4 text-3xl font-medium text-muted">
                        /
                      </p>
                      <Link
                        to="/dashboard/projects/$projectId"
                        params={{ projectId: props.project.id }}
                        className="font-medium"
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
              <Button variant="ghost" size="icon" asChild>
                <a href="https://github.com/Xenfo/adrastos" target="_blank">
                  <SiGithub className="h-4 w-4" />
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
          <Button variant="ghost" size="sm" asChild>
            <Link
              to="/dashboard/projects/$projectId"
              params={{ projectId: props.project.id }}
              activeOptions={{ exact: true }}
              className="text-muted-foreground hover:bg-accent/70 data-[status=active]:bg-accent data-[status=active]:text-primary"
            >
              Overview
            </Link>
          </Button>
          <Button variant="ghost" size="sm" asChild>
            <Link
              to="/dashboard/projects/$projectId/auth"
              params={{ projectId: props.project.id }}
              className="text-muted-foreground hover:bg-accent/70 data-[status=active]:bg-accent data-[status=active]:text-primary"
            >
              Auth
            </Link>
          </Button>
          <Button variant="ghost" size="sm" asChild>
            <Link
              to="/dashboard/projects/$projectId/tables"
              params={{ projectId: props.project.id }}
              className="text-muted-foreground hover:bg-accent/70 data-[status=active]:bg-accent data-[status=active]:text-primary"
            >
              Tables
            </Link>
          </Button>
          <Button variant="ghost" size="sm" asChild>
            <Link
              to="/dashboard/projects/$projectId/storage"
              params={{ projectId: props.project.id }}
              className="text-muted-foreground hover:bg-accent/70 data-[status=active]:bg-accent data-[status=active]:text-primary"
            >
              Storage
            </Link>
          </Button>
          <Button variant="ghost" size="sm" asChild>
            <Link
              to="/dashboard/projects/$projectId/settings"
              params={{ projectId: props.project.id }}
              className="text-muted-foreground hover:bg-accent/70 data-[status=active]:bg-accent data-[status=active]:text-primary"
            >
              Settings
            </Link>
          </Button>
        </div>
      ) : (
        'teamId' in props && (
          <div className="space-x-1">
            <Button variant="ghost" size="sm" asChild>
              <Link
                to="/dashboard/teams/$teamId"
                params={{ teamId: props.teamId }}
                activeOptions={{ exact: true }}
                className="text-muted-foreground hover:bg-accent/70 data-[status=active]:bg-accent data-[status=active]:text-primary"
              >
                Projects
              </Link>
            </Button>
            <Button variant="ghost" size="sm" asChild>
              <Link
                to="/dashboard/teams/$teamId/settings"
                params={{ teamId: props.teamId }}
                activeOptions={{ exact: true }}
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
