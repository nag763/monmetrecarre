import logging
import os
import sys
import tempfile
import urllib.request
import zipfile

import boto3
from awsglue.utils import getResolvedOptions
import pandas as pd
import io

# --- Logging setup ---
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s [%(levelname)s] %(message)s"
)
logger = logging.getLogger(__name__)

# --- Glue job parameters ---
args = getResolvedOptions(
    sys.argv, ["bucket_name", "compressed_prefix", "uncompressed_prefix", "TempDir"]
)

bucket_name = args["bucket_name"]
compressed_prefix = args["compressed_prefix"]
uncompressed_prefix = args["uncompressed_prefix"]
temp_dir = args["TempDir"]

# --- Sanitize bucket name ---
if bucket_name.startswith("arn:aws:s3:::"):
    bucket_name = bucket_name.replace("arn:aws:s3:::", "")
logger.info(f"Using bucket: {bucket_name}")

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
    year = dataset["year"]
    uri = dataset["uri"]

    s3_zip_key = f"{temp_dir}/donnees_foncieres_{year}.zip"
    s3_partition_path = f"{uncompressed_prefix}/year={year}/"

    # Skip already processed data
    try:
        response = s3_client.list_objects_v2(
            Bucket=bucket_name, Prefix=s3_partition_path, MaxKeys=1
        )
        if response.get("KeyCount", 0) > 0:
            logger.info(
                f"Data for year {year} already exists in {s3_partition_path}. Skipping."
            )
            continue
    except Exception as e:
        logger.warning(f"Could not check existing data for {year}: {e}")

    logger.info(f"Processing year {year}...")
    logger.info(f"Downloading from {uri}")

    # --- Download the ZIP to a temporary file instead of memory ---
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
            s3_key = f"{s3_partition_path}{file_name}"

            with zip_ref.open(file_info) as extracted_file:
                
                # Read CSV into pandas DataFrame
                df = pd.read_csv(extracted_file, delimiter='|', header=0, low_memory=False)
                
                # Create buffer for parquet file
                parquet_buffer = io.BytesIO()
                
                # Write DataFrame to parquet format in memory
                df.to_parquet(parquet_buffer)
                
                # Reset buffer position
                parquet_buffer.seek(0)
                
                # Change file extension to .parquet
                s3_key = s3_key.replace('.txt', '.parquet')
                
                # Upload parquet file to S3
                s3_client.upload_fileobj(parquet_buffer, bucket_name, s3_key)

            logger.info(f"Extracted and uploaded: {s3_key}")

    logger.info(f"Successfully processed year {year}.")


logger.info("--- All datasets processed successfully. ---")
