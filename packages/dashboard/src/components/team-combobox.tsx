import { zodResolver } from '@hookform/resolvers/zod';
import { Link } from '@tanstack/react-router';
import { Check, ChevronsUpDown, PlusCircle } from 'lucide-react';
import { useMemo, useState } from 'react';
import { useForm } from 'react-hook-form';
import { z } from 'zod';

import {
  Button,
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
  Dialog,
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
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '~/components';
import { useCreateTeamMutation } from '~/hooks';
import { cn } from '~/lib';
import { Team } from '~/types';

const formSchema = z.object({
  name: z.string().min(1, { message: 'Name is required' }),
});

export const TeamCombobox: React.FC<{ teams: Team[]; teamId: string }> = ({
  teams,
  teamId,
}) => {
  const [open, setOpen] = useState(false);
  const [isOpen, setIsOpen] = useState(false);
  const [value, setValue] = useState(
    teams.find((team) => team.id === teamId)?.name.toLowerCase() ?? '',
  );

  const team = useMemo(
    () => teams.find((team) => team.name.toLowerCase() === value)!,
    [teams, value],
  );

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      name: '',
    },
  });

  const { mutateAsync } = useCreateTeamMutation();

  return (
    <div className="flex flex-row items-center">
      <Link
        to="/dashboard/teams/$teamId"
        params={{ teamId: team.id }}
        className="mr-1 font-medium"
      >
        {teams.find((team) => team.name.toLowerCase() === value)?.name}
      </Link>

      <Dialog open={isOpen} onOpenChange={setIsOpen}>
        <Popover open={open} onOpenChange={setOpen}>
          <PopoverTrigger asChild>
            <Button
              variant="outline"
              role="combobox"
              aria-expanded={open}
              className="border-0 px-2"
            >
              <ChevronsUpDown className="h-4 w-4 shrink-0 opacity-50" />
            </Button>
          </PopoverTrigger>
          <PopoverContent
            className="w-[250px] p-0"
            align="end"
            alignOffset={-65}
          >
            <Command>
              <CommandInput placeholder="Search teams..." />
              <CommandList>
                <CommandEmpty>No team found.</CommandEmpty>

                <CommandGroup>
                  {teams.map((team) => (
                    <Link
                      key={team.id}
                      to="/dashboard/teams/$teamId"
                      params={{ teamId: team.id }}
                    >
                      <CommandItem
                        value={team.name}
                        onSelect={(currentValue) => {
                          setValue(currentValue);
                          setOpen(false);
                        }}
                      >
                        <Check
                          className={cn(
                            'mr-2 h-4 w-4',
                            value === team.name.toLowerCase()
                              ? 'opacity-100'
                              : 'opacity-0',
                          )}
                        />
                        {team.name}
                      </CommandItem>
                    </Link>
                  ))}
                </CommandGroup>

                <CommandSeparator />

                <CommandGroup>
                  <DialogTrigger className="w-full">
                    <CommandItem
                      onSelect={() => setOpen(false)}
                      onClick={() => undefined}
                    >
                      <PlusCircle className="mr-2 h-4 w-4" /> Create new team
                    </CommandItem>
                  </DialogTrigger>
                </CommandGroup>
              </CommandList>
            </Command>
          </PopoverContent>
        </Popover>

        <DialogContent>
          <DialogHeader>
            <DialogTitle>Create a new team</DialogTitle>
          </DialogHeader>

          <Form {...form}>
            <form
              onSubmit={(e) =>
                void form.handleSubmit(async (values) => {
                  await mutateAsync(values.name);
                  setIsOpen(false);
                })(e)
              }
            >
              <div className="space-y-5">
                <FormField
                  control={form.control}
                  name="name"
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

                <DialogFooter>
                  <Button type="submit">Submit</Button>
                </DialogFooter>
              </div>
            </form>
          </Form>
        </DialogContent>
      </Dialog>
    </div>
  );
};
