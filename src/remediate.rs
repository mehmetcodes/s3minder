use super::gather;
use log::{trace,info, warn,debug,error};
use std::default::Default;
use std::error::Error;
use handlebars::Handlebars;
use rusoto_s3::{S3, S3Client, ServerSideEncryptionConfiguration, ServerSideEncryptionRule, ServerSideEncryptionByDefault, 
  PutBucketEncryptionRequest, GetBucketEncryptionRequest, GetBucketLifecycleRequest,GetBucketLocationRequest,
  HeadObjectRequest,CopyObjectRequest,ListObjectsRequest,GetBucketWebsiteRequest };
use rusoto_core::{Region};
#[macro_use]
use serde_json::json;

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
            applykmskey: false,
        }
    }
}

#[derive(Debug,Clone,Serialize)]
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

  pub fn transit_policy_template(bucket:&String)->Result<String, Box<dyn Error>>{
    let mut reg = Handlebars::new();
    let default_transit_policy = r###"
    {
      "Id": "ExamplePolicy",
      "Version": "2012-10-17",
      "Statement": [
        {
          "Sid": "AllowSSLRequestsOnly",
          "Action": "s3:*",
          "Effect": "Deny",
          "Resource": [
            "arn:aws:s3:::{{bucket}}",
            "arn:aws:s3:::{{bucket}}/*"
          ],
          "Condition": {
             "Bool": {
              "aws:SecureTransport": "false"
            }
          },
          "Principal": "*"
        }
      ]
    }
    "###;
    // render without register
    
    let result = reg.render_template(default_transit_policy, &json!({"bucket": bucket}))?;
    Ok(result)
  }
  
  
  pub fn sse_policy_template()->Result<String, Box<dyn Error>>{
    let default_sse_policy = r###"
  <ServerSideEncryptionConfiguration xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
    <Rule>
       <ApplyServerSideEncryptionByDefault>
          <SSEAlgorithm>AES256</SSEAlgorithm>
       </ApplyServerSideEncryptionByDefault>
    </Rule>
  </ServerSideEncryptionConfiguration>"###;
    
    Ok(String::from(default_sse_policy))
  
  }
  


pub async fn apply_sse_encryption_rule( s3_client:&S3Client ,bucket:&String){
  let sse_rules_vector = vec![ServerSideEncryptionRule{  apply_server_side_encryption_by_default:Some(ServerSideEncryptionByDefault{ sse_algorithm:"AES256".to_string(),kms_master_key_id: None,..Default::default() }),..Default::default()}];
  let pber = PutBucketEncryptionRequest{ bucket:bucket.to_string(), server_side_encryption_configuration:ServerSideEncryptionConfiguration{rules:sse_rules_vector,..Default::default()},..Default::default() };
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
    debug!("Remedy is {:?}",remedy);
    for b in super::gather::BUCKET_LIST.lock().unwrap().values(){
      debug!("Bucket {:?}",b);
      if remedy.skipwebbuckets && !b.web_bucket && !b.default_encryption 
      { 
          //Then check if we are applying default sse encryption or custom encryption
          if remedy.applykmskey
          {
            apply_default_kms_encryption_rule(s3_client, &b.bucket_name).await;
          }else if remedy.applysseencryption{
            apply_sse_encryption_rule( s3_client ,&b.bucket_name ).await;
            copy_in_place_items_in_bucket(s3_client,&b.bucket_name).await;
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


async fn copy_object_in_place(client: &S3Client, bucket: &str, filename: &str) {
  let req = CopyObjectRequest {
      bucket: bucket.to_owned(),
      key: filename.to_owned(),
      copy_source: format!("{}/{}", bucket, filename),
      content_type: Some("application/json".to_owned()),
      metadata_directive: Some("REPLACE".to_owned()),
      server_side_encryption: Some("AES256".to_owned()),
      ..Default::default()
  };
  let result = client
      .copy_object(req)
      .await;
  match result{
    Ok(_)=>{},
    Err(err)=>{
      error!("Couldn't copy object from {} - {}",bucket,filename);
      error!("{:?}",err);
    }
  }
  
}

pub async fn copy_in_place_items_in_bucket(client: &S3Client, bucket: &str) {
  let mut list_request = ListObjectsRequest {
      delimiter: Some("/".to_owned()),
      bucket: bucket.to_owned(),
      max_keys: Some(1000),
      ..Default::default()
  };

  // Add error handling here, seems to crash in specific
  let mut response = client
        .list_objects(list_request.clone())
        .await;
  match response{
    Ok(resy)=>{
      let mut res = resy;
      loop{
        debug!("Args: bucket {}",bucket.to_owned());
        let contents1 = res.contents;
        match contents1{
          Some(_)=> {
            for obj in contents1.unwrap().iter(){
              trace!("{}", obj.key.as_ref().unwrap());
              //copy_object(client, bucket, obj.key.as_ref().unwrap()).await;
              copy_object_in_place(client, bucket,  &obj.key.clone().unwrap().to_string().to_owned()).await;
              info!("Copied {} in place", obj.key.clone().unwrap().to_string().to_owned());
              let head_check_encryption = HeadObjectRequest{ bucket:bucket.to_owned(), key: obj.key.clone().unwrap().to_string().to_owned(),..Default::default() };
              let head_result =  client.head_object(head_check_encryption).await.expect("Failed to retrieve head for object");
              debug!("{:#?}",head_result);
              match head_result.ssekms_key_id{
                Some(key_id)=>{
                    trace!("Encrypted by KMS key {} {}",key_id,obj.key.clone().unwrap().as_str() );
                },
                _=>{
                   match head_result.server_side_encryption{
                     Some(algo)=>{
                          trace!("Encrypted by SSE-C using {}",algo);      
                     }
                     _=>{
                      trace!("Not encrypted");     
                    }
                  }
                }
              }
            }
          },
          _ =>{
                trace!("No objects found");
          }
        }
        match res.next_marker {
            Some(_)=>{
    
            },
            _=>{
              debug!("No further pages of objects");
              break;
            }
        }
        list_request.marker = Some(res.next_marker.unwrap());
        list_request.max_keys = Some(1000);
        res = client
            .list_objects(list_request.clone())
            .await
            .expect("list objects failed");
      }

    },
    Err(err)=>{
      error!("Couldn't list objects");
      error!("{:?}",err);
    }
  }
  
}