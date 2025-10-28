export const handler = async (event) => {

  const expectedSecret = process.env.SECRET_HEADER_VALUE;

  const receivedSecret = event?.headers['x-origin-verify'];

  if (!expectedSecret || !receivedSecret) {
    return { "isAuthorized": false };
  }

  return { "isAuthorized": receivedSecret === expectedSecret };
};
