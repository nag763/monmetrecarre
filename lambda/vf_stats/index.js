const { DynamoDBClient } = require("@aws-sdk/client-dynamodb");
const { DynamoDBDocumentClient, QueryCommand } = require("@aws-sdk/lib-dynamodb");
const { z } = require("zod");

const codeInseeSchema = z().length(5);

const headers = { 'Content-Type': 'application/json' };

const produceMessage = ({message, statusCode=400}) => {
  console.warn(message);
  return { statusCode: statusCode, body: JSON.stringify({message}), isBase64Encoded: false, headers};
}

exports.handler = async function (event) {

  const { codeInsee } = event.pathParameters;
  if(!codeInsee) {
    return produceMessage({message: "Bad Request: Missing codeInsee in path parameter"});
  }
  const codeInseeValidation = codeInseeSchema.safeParse(codeInsee);
  if (!codeInseeValidation.success) {

    return produceMessage({message: "Bad Request: Invalid codeInsee in path parameter"});
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
    return produceMessage({statusCode: 204, message: "Bad Request: Invalid codeInsee in path parameter"});
  }
  return {
    statusCode: 200,
    body: JSON.stringify(response.Items),
    isBase64Encoded: false,
    headers
  };
};