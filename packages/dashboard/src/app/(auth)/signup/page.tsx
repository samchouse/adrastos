import { Card, CardHeader, CardTitle } from '~/components';

import { SignupForm } from './form';

const Page: React.FC = () => (
  <div className="flex h-full w-screen items-center justify-center">
    <Card className="mx-6 w-full sm:m-0 sm:w-[500px]">
      <CardHeader>
        <CardTitle>Sign Up</CardTitle>
      </CardHeader>

      <SignupForm />
    </Card>
  </div>
);

export default Page;
