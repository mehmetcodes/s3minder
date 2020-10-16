aws s3api put-bucket-encryption --bucket bucketmctest2 --server-side-encryption-configuration "{ \"Rules\": [{ \"ApplyServerSideEncryptionByDefault\": {\"SSEAlgorithm\": \"AES256\"} }] }"
aws s3api put-bucket-lifecycle-configuration  --bucket bucketname --lifecycle-configuration file://lifecycle.json 
