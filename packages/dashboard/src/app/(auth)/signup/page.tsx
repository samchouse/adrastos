import { Card, CardHeader, CardTitle, SignupForm } from '~/components';

const Page = () => (
  <main>
    <div className="flex h-screen w-screen items-center justify-center">
      <Card className="max-w-[500px] lg:min-w-[500px]">
        <CardHeader>
          <CardTitle>Sign Up</CardTitle>
        </CardHeader>

        <SignupForm />
      </Card>
    </div>
  </main>
);

export default Page;
