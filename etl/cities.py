import requests
import sys

from awsglue.utils import getResolvedOptions
import boto3

args = getResolvedOptions(
    sys.argv, ["dynamodb_table_name"]
)

dynamodb_table_name = args["dynamodb_table_name"]

URL = "https://geo.api.gouv.fr/communes/"

res = requests.get(URL)

if res.status_code == 200:
    dynamodb = boto3.resource("dynamodb")
    table = dynamodb.Table(dynamodb_table_name)
    with table.batch_writer() as batch:
        cities = res.json()
        for city in cities:
            batch.put_item(Item=city)