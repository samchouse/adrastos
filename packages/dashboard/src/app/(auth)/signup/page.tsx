import Link from 'next/link';

import {
  Button,
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
  SignupForm
} from '~/components';

const Page = () => (
  <main>
    <div className="flex h-screen w-screen items-center justify-center">
      <Card className="max-w-[500px] lg:min-w-[500px]">
        <CardHeader>
          <CardTitle>Sign Up</CardTitle>
        </CardHeader>
        <CardContent>
          <SignupForm />
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
