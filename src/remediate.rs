use super::gather;
use log::{trace,info, warn,debug,error};
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
      info!("Default Encryption applied to bucket {}", bucket);
    },
    Err(e)=>{ error!("bucket {} has an error\n{:#?}",bucket,e)},
    _=>{ error!("Something unexpected happened");},
  }
  
}

pub async fn apply_default_kms_encryption_rule( s3_client:&S3Client ,bucket:&String ){

}

  

pub async fn remediate_buckets( s3_client:&S3Client,remedy:S3RemediateOptions ){
    let count = super::gather::BUCKET_LIST.lock().unwrap().values().len();
    info!("Buckets found to determine remediation: {}",count);
    for b in super::gather::BUCKET_LIST.lock().unwrap().values(){
      if remedy.skipwebbuckets && !b.web_bucket && !b.default_encryption 
      { 
          //Then check if we are applying default sse encryption or custom encryption
          if remedy.applykmskey
          {
            apply_default_kms_encryption_rule(s3_client, &b.bucket_name).await;
          }else if remedy.applysseencryption{
            apply_sse_encryption_rule( s3_client ,&b.bucket_name ).await;
          }else{
            //Default encryption skipped
          }
           
        
         
      }else{
        info!("Encryption skipped: Skipped encryption for {:?} because default encryption: {}, web_bucket {}",b.bucket_name,b.default_encryption,b.web_bucket );
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


