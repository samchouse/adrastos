import Link from 'next/link';

import {
  Button,
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
  Checkbox,
  Input,
  Label
} from '~/components';

const Page = () => (
  <main>
    <div className="flex h-screen w-screen items-center justify-center">
      <Card className="w-[400px]">
        <CardHeader>
          <CardTitle>Sign Up</CardTitle>
        </CardHeader>
        <CardContent>
          <form>
            <div className="flex flex-col gap-1">
              <div>
                <Label htmlFor="email">Email</Label>
                <Input id="email" type="email" placeholder="Email" />
              </div>
              <div>
                <Label htmlFor="password">Password</Label>
                <Input id="password" type="password" placeholder="Password" />
              </div>
              <div className="mt-3 flex items-center space-x-2">
                <Checkbox id="tac" />
                <Label htmlFor="tac">I accept the Terms and Conditions</Label>
              </div>
            </div>
          </form>
        </CardContent>
        <CardFooter>
          <div className="w-full">
            <Button className="w-full">Sign Up</Button>

            <Link
              href="/login"
              className="text-muted-foreground mt-5 block text-center text-sm underline underline-offset-4"
            >
              Already have an account? Login
            </Link>
          </div>
        </CardFooter>
      </Card>
    </div>
  </main>
);

export default Page;
