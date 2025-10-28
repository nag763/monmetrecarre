const produceError = (error) => {
    console.warn(error)
    return {
      isAuthorized: false,
      context: {
        reason: error,
      },
    };
}

export const handler = async (event) => {

  const expectedSecret = process.env.SECRET_HEADER_VALUE;

  if(!expectedSecret) {
    return produceError("No secret found in Authorizer");
  }

  const token = event.headers?.['x-origin-verify'] || event.headers?.['X-Origin-Verify'];

  if (!token) {
    return produceError("Invalid or missing token");
  }

  if (token !== expectedSecret) {
    return produceError("Token value is not matching secret");
  }

  return { "isAuthorized": true };
};
