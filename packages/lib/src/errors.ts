export class ResponseError extends Error {
  constructor(
    message: string,
    public details: {
      success: false;
      status: string;
      error?: string;
      message?: string;
      errors?: Record<string, unknown>;
    },
  ) {
    super(message);
    this.name = 'ResponseError';
  }
}
