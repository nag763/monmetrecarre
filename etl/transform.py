import json
import sys
from decimal import Decimal

import boto3
from awsglue.context import GlueContext
from awsglue.job import Job
from awsglue.utils import getResolvedOptions
from pyspark.context import SparkContext
from pyspark.sql import functions as F
from pyspark.sql.functions import col, regexp_replace


def convert_floats_to_decimal(item):
    for k, v in item.items():
        if isinstance(v, float):
            item[k] = Decimal(str(v))  # convert via string to avoid precision issues
    return item


# --- Initialize GlueContext and Spark ---
sc = SparkContext()
glueContext = GlueContext(sc)
spark = glueContext.spark_session
job = Job(glueContext)

# --- Parse Job Parameters ---
args = getResolvedOptions(
    sys.argv, ["JOB_NAME", "glue_database", "glue_table", "dynamodb_table_name"]
)

job_name = args["JOB_NAME"]
glue_database = args["glue_database"]
glue_table = args["glue_table"]
dynamodb_table_name = args["dynamodb_table_name"]

job.init(job_name, args)

# --- Read from Glue Catalog ---
dynamic_frame = glueContext.create_dynamic_frame.from_catalog(
    database=glue_database,
    table_name=glue_table,
    transformation_ctx="datasource_uncompressed",
)
df = dynamic_frame.toDF()

# --- Transformations ---
df = df.withColumn("year", col("year").cast("integer"))
df = df.withColumn("month", col("month").cast("integer"))
df = df.withColumn(
    "valeur_fonciere",
    regexp_replace(col("valeur_fonciere"), ",", ".").cast("decimal(18,2)"),
)
df = df.withColumn(
    "surface_reelle_bati", col("surface_reelle_bati").cast("decimal(18,2)")
)

filtered_df = df.filter(col("type_local").isin(["Appartement", "Maison"]))

df_with_individual_price = filtered_df.filter(
    col("surface_reelle_bati").isNotNull() & (col("surface_reelle_bati") > 0)
).withColumn("individual_prix_m2", col("valeur_fonciere") / col("surface_reelle_bati"))

# Aggregations
grouped = (
    df_with_individual_price.groupBy("year", "month", "code_insee", "type_local")
    .agg(
        F.avg("valeur_fonciere").alias("avg_valeur_fonciere"),
        F.avg("surface_reelle_bati").alias("avg_surface_bati"),
        F.avg("surface_terrain").alias("avg_surface_terrain"),
        F.count("*").alias("count"),
        F.percentile_approx("individual_prix_m2", 0.95).alias("prix_metre_carre_95p"),
        F.percentile_approx("individual_prix_m2", 0.05).alias("prix_metre_carre_05p"),
        F.percentile_approx("individual_prix_m2", 0.75).alias("prix_metre_carre_75p"),
        F.percentile_approx("individual_prix_m2", 0.25).alias("prix_metre_carre_25p"),
    )
    .filter(5 < col("count"))
)

grouped = (
    grouped.withColumn(
        "prix_metre_carre_avg",
        F.round(col("avg_valeur_fonciere") / col("avg_surface_bati"), 2),
    )
    .withColumn("avg_valeur_fonciere", F.round(col("avg_valeur_fonciere"), 2))
    .withColumn("avg_surface_bati", F.round(col("avg_surface_bati"), 2))
)

# --- Flatten for DynamoDB ---
# PK = Commune, SK = Type#Year
flattened_df = grouped.withColumn(
    "type_year", F.concat_ws("_", col("type_local"), col("year"), col("month"))
)

# Convert to JSON strings for DynamoDB
items = (
    flattened_df.toJSON().collect()
)  # Caution: collect() can be large for big datasets

# --- Write to DynamoDB ---
dynamodb = boto3.resource("dynamodb")
table = dynamodb.Table(dynamodb_table_name)

print(f"Writing {len(items)} items to DynamoDB table {dynamodb_table_name}...")

with table.batch_writer() as batch:
    for item_str in items:
        item = json.loads(item_str)
        item = convert_floats_to_decimal(item)
        batch.put_item(Item=item)

print("✅ DynamoDB write complete.")
job.commit()
