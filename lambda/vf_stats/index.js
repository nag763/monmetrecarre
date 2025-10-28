const { DynamoDBClient } = require("@aws-sdk/client-dynamodb");
const { DynamoDBDocumentClient, QueryCommand } = require("@aws-sdk/lib-dynamodb");
const { z } = require("zod");

const codeInseeSchema = z.string().length(5);

exports.handler = async function (event) {

  const { codeInsee } = event.pathParameters;
  if(!codeInsee) {
    return { statusCode: 400, body: {message: "Bad Request: Missing code_insee in path parameter"}, isBase64Encoded: false };
  }
  const codeInseeValidation = codeInseeSchema.safeParse(codeInsee);
  if (!codeInseeValidation.success) {
    return { statusCode: 400, body: "Bad Request: Invalid codeInsee in path parameter" };
  }

  const ddbClient = new DynamoDBClient({ region: process.env.AWS_REGION || "eu-central-1" });
  const ddbDocClient = DynamoDBDocumentClient.from(ddbClient);
  const input = {
    ExpressionAttributeValues: {
      ":code_insee": codeInsee,
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
    body: JSON.stringify(response.Items),
    isBase64Encoded: false
  };
};