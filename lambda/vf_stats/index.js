const { DynamoDBClient } = require("@aws-sdk/client-dynamodb");
const { DynamoDBDocumentClient, QueryCommand } = require("@aws-sdk/lib-dynamodb");
const { z } = require("zod");

bodySchema = z.object({
  codeInsee: z.string().length(5)
})

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

  const bodyParsed = bodySchema.safeParse(requestBody);
  if (!bodyParsed.success) {
    return { statusCode: 400, body: "Bad Request: Invalid codeInsee format" };
  }

  const ddbClient = new DynamoDBClient({ region: process.env.AWS_REGION || "eu-central-1" });
  const ddbDocClient = DynamoDBDocumentClient.from(ddbClient);
  const input = {
    ExpressionAttributeValues: {
      ":code_insee": bodyParsed.data.codeInsee,
    },
    KeyConditionExpression: "code_insee = :code_insee",
    TableName: process.env.DYNAMODB_TABLE_NAME,
  };
  const command = new QueryCommand(input);
  const response = await ddbDocClient.send(command);

  if (!response.Items || response.Items.length === 0) {
    return { statusCode: 204, body: "Not Found: No data for the given code_insee" };
  }
  return {
    statusCode: 200,
    body: response.Items,
  };
};