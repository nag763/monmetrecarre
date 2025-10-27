const { DynamoDBClient, QueryCommand } = require("@aws-sdk/client-dynamodb");

exports.handler = async function (event) {

  let requestBody;

  if (!event.body) {
    return { statusCode: 400, body: "Bad Request: Missing body" };
  }

  try {
    if (event.isBase64Encoded) {
      const decodedBody = Buffer.from(event.body, 'base64').toString('utf-8');
      requestBody = JSON.parse(decodedBody);
    } else {
      requestBody = JSON.parse(event.body);
    }
  } catch (error) {
    return { statusCode: 400, body: error.message };
  }

  const code_insee = requestBody.code_insee;
  if (!code_insee) {
    return { statusCode: 400, body: "Bad Request: Missing code_insee" };
  }

  const ddbClient = new DynamoDBClient({ region: process.env.AWS_REGION || "eu-central-1" });
  const input = {
    ExpressionAttributeValues: {
      ":v1": { S: code_insee },
    },
    KeyConditionExpression: "code_insee = :v1",
    TableName: process.env.DYNAMODB_TABLE_NAME,
  };
  const command = new QueryCommand(input);
  const response = await ddbClient.send(command);

  if (!response.Items || response.Items.length === 0) {
    return { statusCode: 204, body: "Not Found: No data for the given code_insee" };
  }
  return {
    statusCode: 200,
    body: response.Items,
  };
};