import { NextResponse, type NextRequest } from 'next/server';

export const middleware = (request: NextRequest) => {
  if (
    request.nextUrl.pathname === '/login' &&
    request.nextUrl.searchParams.has('to')
  ) {
    const response = NextResponse.next();
    response.cookies.delete('isLoggedIn');
    return response;
  }

  console.log(request.cookies.get('isLoggedIn'), request.nextUrl.pathname);

  if (
    ['/', '/login', '/signup'].includes(request.nextUrl.pathname) &&
    request.cookies.get('isLoggedIn')?.value === 'true'
  )
    return NextResponse.redirect(new URL('/dashboard', request.url));

  if (
    request.nextUrl.pathname.startsWith('/dashboard') &&
    request.cookies.get('isLoggedIn')?.value !== 'true'
  )
    return NextResponse.redirect(new URL('/', request.url));
};
