import logging
import os
import re
import sys
import tempfile
import urllib.request
import zipfile

import boto3
import pandas as pd
import pyspark.sql.functions as F  # safer than importing individual functions
from awsglue.utils import getResolvedOptions
from pyspark.sql import SparkSession


def to_snake_case(s):
    """
    Converts a string to snake_case.

    - Handles camelCase and PascalCase by inserting an underscore before uppercase letters.
    - Replaces hyphens and spaces with underscores.
    - Handles consecutive non-alphanumeric characters.
    - Converts the entire string to lowercase.
    """
    # 1. Insert underscores before capital letters in camel/PascalCase
    s = re.sub(r"(.)([A-Z][a-z]+)", r"\1_\2", s)
    s = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", s)

    # 2. Replace any sequence of non-alphanumeric characters (like ' ', '-', '__') with a single '_'
    s = re.sub(r"[^a-zA-Z0-9]+", "_", s)

    # 3. Convert to lowercase and remove any leading/trailing underscores
    return s.lower().strip("_")


# --- Logging setup ---
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s [%(levelname)s] %(message)s"
)
logger = logging.getLogger(__name__)

# --- Glue job parameters ---
args = getResolvedOptions(
    sys.argv,
    [
        "bucket_name",
        "compressed_prefix",
        "uncompressed_prefix",
        "TempDir",
        "temp_dir_path",
    ],
)

bucket_name = args["bucket_name"]
compressed_prefix = args["compressed_prefix"]
uncompressed_prefix = args["uncompressed_prefix"]
temp_dir = args["TempDir"]
temp_dir_path = args["temp_dir_path"]

# --- Sanitize bucket name ---
if bucket_name.startswith("arn:aws:s3:::"):
    bucket_name = bucket_name.replace("arn:aws:s3:::", "")
logger.info(f"Using bucket: {bucket_name}")

# --- Spark session ---
spark = SparkSession.builder.getOrCreate()

# --- Initialize S3 client ---
s3_client = boto3.client("s3")

# --- Datasets to process ---
datasets = [
    {
        "uri": "https://www.data.gouv.fr/api/1/datasets/r/4d741143-8331-4b59-95c2-3b24a7bdbe3c",
        "year": "2025",
    },
    {
        "uri": "https://www.data.gouv.fr/api/1/datasets/r/af812b0e-a898-4226-8cc8-5a570b257326",
        "year": "2024",
    },
    {
        "uri": "https://www.data.gouv.fr/api/1/datasets/r/cc8a50e4-c8d1-4ac2-8de2-c1e4b3c44c86",
        "year": "2023",
    },
    {
        "uri": "https://www.data.gouv.fr/api/1/datasets/r/8c8abe23-2a82-4b95-8174-1c1e0734c921",
        "year": "2022",
    },
]

logger.info(f"--- Starting ZIP-to-S3 process for bucket: {bucket_name} ---")

# --- Process each dataset ---
for dataset in datasets:
    dataset_year = dataset["year"]  # rename to avoid conflict with F.year()
    uri = dataset["uri"]

    s3_zip_key = f"{compressed_prefix}/donnees_foncieres_{dataset_year}.zip"

    logger.info(f"Processing year {dataset_year}...")
    logger.info(f"Downloading from {uri}")

    # --- Download the ZIP to a temporary file ---
    with tempfile.NamedTemporaryFile(delete=False) as tmp_zip_file:
        with urllib.request.urlopen(uri) as response:
            CHUNK = 16 * 1024
            while True:
                chunk = response.read(CHUNK)
                if not chunk:
                    break
                tmp_zip_file.write(chunk)
        tmp_zip_path = tmp_zip_file.name

    # --- Upload the raw ZIP file to S3 ---
    logger.info(f"Uploading raw ZIP to s3://{bucket_name}/{s3_zip_key}")
    s3_client.upload_file(tmp_zip_path, bucket_name, s3_zip_key)

    # --- Extract and upload each file to S3 ---
    with zipfile.ZipFile(tmp_zip_path, "r") as zip_ref:
        for file_info in zip_ref.infolist():
            if file_info.is_dir():
                continue

            file_name = os.path.basename(file_info.filename)
            logger.info(f"Extracting {file_name} from ZIP...")

            # Extract file to temp, then upload to S3
            with tempfile.NamedTemporaryFile(delete=False, suffix=".txt") as tmp_csv:
                with zip_ref.open(file_info) as extracted_file:
                    tmp_csv.write(extracted_file.read())
                tmp_csv_path = tmp_csv.name

            s3_csv_key = f"{temp_dir_path}/{file_name}"
            s3_client.upload_file(tmp_csv_path, bucket_name, s3_csv_key)

    # --- Read CSV with Spark ---
df = spark.read.csv(
    f"s3://{bucket_name}/{temp_dir_path}", sep="|", header=True, inferSchema=True
)

old_columns = df.columns

# Create new snake_case column names
new_columns = [to_snake_case(c) for c in old_columns]

# Apply new names using toDF()
df = df.toDF(*new_columns)

# --- Transform ---
df = (
    df.select(
        "date_mutation",
        "type_local",
        "valeur_fonciere",
        "code_postal",
        "commune",
        "code_departement",
        "code_commune",
        "surface_terrain",
        "surface_reelle_bati",
    )
    .withColumn(
        "code_insee",
        F.concat(F.col("code_departement"), F.lpad(F.col("code_commune"), 3, "0")),
    )
    .withColumn("date_mutation", F.to_date("date_mutation", "dd/MM/yyyy"))
    .withColumn("year", F.year(F.col("date_mutation")))
    .withColumn("month", F.month(F.col("date_mutation")))
    .withColumn("valeur_fonciere", F.regexp_replace(F.col("valeur_fonciere"), ",", "."))
)

# --- Write Parquet ---
output_path = f"s3://{bucket_name}/{uncompressed_prefix}/vf/"
logger.info(f"Writing to {output_path}")
df.write.partitionBy("year", "month").parquet(output_path, mode="append")


logger.info(f"Successfully processed year {dataset_year}.")

logger.info("--- All datasets processed successfully. ---")
