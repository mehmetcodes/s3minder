use super::gather;
use std::default::Default;
use rusoto_s3::{S3, S3Client, ServerSideEncryptionConfiguration, ServerSideEncryptionRule, ServerSideEncryptionByDefault, 
  PutBucketEncryptionRequest, GetBucketEncryptionRequest, GetBucketLifecycleRequest,GetBucketLocationRequest,
  HeadObjectRequest,CopyObjectRequest,ListObjectsRequest,GetBucketWebsiteRequest };
use rusoto_core::{Region};

impl Default for S3RemediateOptions {
    fn default() -> Self {
        S3RemediateOptions {
            encryptobjects: true,
            trackobjects: true,
            skipwebbuckets: true,
            applylifecycle: true,
            applysseencryption: true,
            applytransitpolicy: false,
            customtransitpolicy: false,
            applykmskey: true,
        }
    }
}


pub struct S3RemediateOptions {
    pub encryptobjects: bool,
    pub trackobjects: bool,
    pub skipwebbuckets: bool,
    pub applylifecycle: bool,
    pub applysseencryption: bool,
    pub applytransitpolicy: bool,
    pub customtransitpolicy: bool,
    pub applykmskey: bool,
  }

  pub async fn apply_sse_encryption_rule( s3_client:&S3Client ,bucket:&String){
    let sse_rules_vector = vec![ServerSideEncryptionRule{  apply_server_side_encryption_by_default:Some(ServerSideEncryptionByDefault{ sse_algorithm:"AES256".to_string(),kms_master_key_id: None })}];
    let pber = PutBucketEncryptionRequest{ bucket:bucket.to_string(), server_side_encryption_configuration:ServerSideEncryptionConfiguration{rules:sse_rules_vector} };
    let sse_default_result = s3_client.put_bucket_encryption(pber).await;
    match sse_default_result {
      Ok(r)=>{
        println!("bucket {} has had default encryption applied\n{:#?}",bucket,r);
      },
      Err(e)=>{ println!("bucket {} has an error\n{:#?}",bucket,e)},
      _=>{ println!("Something unexpected happened");},
    }
  }

  

pub async fn remediate_buckets( s3_client:&S3Client,remedy:S3RemediateOptions ){
    for b in super::gather::BUCKET_LIST.lock().unwrap().values(){
      //case skip web buckets
      if !b.default_encryption && !b.web_bucket
      { 
          apply_sse_encryption_rule( s3_client ,&b.bucket_name, ).await; 
          println!("Applied default encryption to non-web bucket {}",b.bucket_name);
      }else{
        println!("Skipped default encryption to bucket {}",b.bucket_name);
        println!("because default_encryption: {} web_bucket {}",b.default_encryption,b.web_bucket);
      }
      if remedy.applylifecycle {
        if !b.web_bucket && remedy.skipwebbuckets {
            //TODO: Apply default encryption

            if remedy.encryptobjects {
                //TODO: Then encrypt the objects 
                if remedy.trackobjects {

                }
            }

        }
      }
      if remedy.applysseencryption || remedy.applykmskey {
        if !b.web_bucket && remedy.skipwebbuckets {
            if remedy.applykmskey {
            
            }else if remedy.applysseencryption {

            }   
        }
      }
      
    }
}


