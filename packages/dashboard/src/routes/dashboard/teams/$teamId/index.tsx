import { zodResolver } from '@hookform/resolvers/zod';
import { Dialog } from '@radix-ui/react-dialog';
import { useSuspenseQueries } from '@tanstack/react-query';
import { Link, createFileRoute } from '@tanstack/react-router';
import { Plus } from 'lucide-react';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';

import {
  Badge,
  Button,
  Card,
  CardFooter,
  CardHeader,
  CardTitle,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
  Input,
} from '~/components';
import { projectsQueryOptions, useCreateProjectMutation } from '~/hooks';

export const Route = createFileRoute('/dashboard/teams/$teamId/')({
  component: TeamIdRoute,
});

const formSchema = z.object({
  name: z.string().min(1, { message: 'Name is required' }),
  hostnames: z.string().min(1, { message: 'Hostnames are required' }),
});

function TeamIdRoute() {
  const { teamId } = Route.useParams();
  const { client } = Route.useRouteContext();

  const [isOpen, setIsOpen] = useState(false);

  const [{ data: projects }] = useSuspenseQueries({
    queries: [projectsQueryOptions(client, teamId)],
  });

  const { mutateAsync } = useCreateProjectMutation(teamId);

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      name: '',
      hostnames: '',
    },
  });

  return (
    <div className="flex w-full flex-col items-center pt-14">
      <div className="flex w-2/4 flex-row justify-between pb-8">
        <h3 className="font-semibold text-3xl">Projects</h3>
        <Dialog open={isOpen} onOpenChange={setIsOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="mr-2 size-4" /> Create new
            </Button>
          </DialogTrigger>
          <DialogContent className="sm:max-w-[425px]">
            <DialogHeader>
              <DialogTitle>Create a project</DialogTitle>
            </DialogHeader>

            <Form {...form}>
              <form
                onSubmit={(e) =>
                  void form.handleSubmit(async (values) => {
                    await mutateAsync({
                      ...values,
                      hostnames: values.hostnames
                        .split(',')
                        .map((h) => h.trim()),
                    });
                    setIsOpen(false);
                    form.reset();
                  })(e)
                }
              >
                <div className="space-y-5">
                  <div className="space-y-1">
                    <FormField
                      name="name"
                      control={form.control}
                      render={({ field }) => (
                        <FormItem className="w-full">
                          <FormLabel>Name</FormLabel>
                          <FormControl>
                            <Input placeholder="Name" {...field} />
                          </FormControl>
                          <FormMessage />
                        </FormItem>
                      )}
                    />
                    <FormField
                      name="hostnames"
                      control={form.control}
                      render={({ field }) => (
                        <FormItem className="w-full">
                          <FormLabel>Hostnames</FormLabel>
                          <FormControl>
                            <Input placeholder="Hostnames" {...field} />
                          </FormControl>
                          <FormMessage />
                        </FormItem>
                      )}
                    />
                  </div>

                  <DialogFooter>
                    <Button
                      type="button"
                      variant="outline"
                      onClick={() => {
                        setIsOpen(false);
                        form.reset();
                      }}
                    >
                      Cancel
                    </Button>
                    <Button type="submit">Submit</Button>
                  </DialogFooter>
                </div>
              </form>
            </Form>
          </DialogContent>
        </Dialog>
      </div>

      <div className="grid w-2/4 grid-cols-2 gap-6">
        {projects.map((project) => (
          <Link
            key={project.id}
            params={{ projectId: project.id }}
            to="/dashboard/projects/$projectId"
          >
            <Card className="flex h-48 flex-col justify-between">
              <CardHeader>
                <CardTitle className="text-xl">{project.name}</CardTitle>
              </CardHeader>

              <CardFooter className="space-x-1">
                {project.hostnames.map((hostname) => (
                  <Badge
                    key={hostname}
                    variant="secondary"
                    className="hover:bg-secondary"
                  >
                    {hostname}
                  </Badge>
                ))}
              </CardFooter>
            </Card>
          </Link>
        ))}
      </div>
    </div>
  );
}
