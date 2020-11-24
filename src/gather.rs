extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_credential;
extern crate lazy_static;
extern crate handlebars;
extern crate serde_json;

extern crate csv;



use std::fs::File;
use std::error::Error;
use handlebars::Handlebars;
use rusoto_s3::{S3, S3Client, ServerSideEncryptionConfiguration, ServerSideEncryptionRule, ServerSideEncryptionByDefault, 
                PutBucketEncryptionRequest, GetBucketEncryptionRequest, GetBucketLifecycleRequest,GetBucketLocationRequest,
                HeadObjectRequest,CopyObjectRequest,ListObjectsRequest,GetBucketWebsiteRequest };
use rusoto_core::{Region};
use std::{fmt,collections::HashMap,sync::Mutex,fs::OpenOptions};
use lazy_static::lazy_static;
use csv::{Writer,Reader};
#[macro_use]
use serde_json::json;
//use csv::{Writer,Reader};

/// Sets debug printouts to give details  
pub static mut DEBUG:bool = false;

/// Sets verbose printouts to give details about the results
pub static mut VERBOSE:bool = false;
  


lazy_static! {
  pub static ref BUCKET_LIST:Mutex< HashMap<String,BucketMeta> > = Mutex::new( 
                    HashMap::new()
                );

}



///
/// 
/// 
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




#[derive(Debug,Clone,Default,Serialize)]
pub struct BucketMeta {
  bucket_name: String,
  bucket_endpoint: String,
  contains_lifecycle: bool,
  default_encryption: bool,
  contains_transit_policy:bool,
  web_bucket:bool,
  objects_checked:bool,

}

fn serialize_bucket_meta(){
  let file = OpenOptions::new()
                                                              .write(true)
                                                              .truncate(true)
                                                              .create(true)
                                                              .open("s3inventory.csv")
                                                              .unwrap();
  let mut csvwriter:csv::Writer<std::fs::File> = csv::WriterBuilder::new()
  .has_headers(true).from_writer( file );
  
  for b in BUCKET_LIST.lock().unwrap().values(){
    let result = csvwriter.serialize(b);
    match result { Ok(r)=>{ println!("{:#?}",r); },Err(r)=>{ println!("{:#?}",r);   } };
  }
}

impl fmt::Display for BucketMeta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n-----===================\n\tendpoint: {},\n\thas lifecycle: {},\n\tdefault encryption: {},\n\tcontains transit policy: {}", self.bucket_name, self.bucket_endpoint,self.contains_lifecycle, self.default_encryption,self.contains_transit_policy)
    }
}



async fn get_bucket_location(b:String) -> BucketMeta {
    let s3_client = S3Client::new(Region::UsWest1);   
    let endpoint_l = s3_client.get_bucket_location( GetBucketLocationRequest{ bucket: b.clone() } ).await;
    let mut meta_bucket:BucketMeta = BucketMeta{ bucket_name: b.clone(), bucket_endpoint: "Error".to_string(), contains_lifecycle: false, default_encryption: false, contains_transit_policy:false,  web_bucket:false, objects_checked:false};
    
    match endpoint_l{
        Ok(val) =>{
          meta_bucket = BucketMeta { 
            bucket_name: b.clone(),  
            bucket_endpoint: ["",b.as_str(),".s3-",&(val.location_constraint.clone().unwrap()),".amazonaws.com"].join("").to_owned(), 
            contains_lifecycle: false,
            default_encryption: false,
            contains_transit_policy: false,
            web_bucket:false, 
            objects_checked:false,
            };
        },
        Err(e) => {
          eprintln!("We got an error{}",e);
        }
        
      }
      return meta_bucket
    }

pub async fn has_bucket_lifecycle( s3_client:&S3Client,bucket:&String )->bool{
  let result = s3_client.get_bucket_lifecycle( GetBucketLifecycleRequest { bucket: bucket.to_string() } ).await; 
  let mut returnvar:bool = false;
  match result{
    Ok(r) => { 
      { 
        unsafe{
          if DEBUG {
              println!("Rules {:?}",r.rules );
          }
        }
      }
      returnvar = true;
    },
    Err(e) => { 
      if  e.to_string().contains("<Code>NoSuchLifecycleConfiguration</Code>")  { 
        unsafe{
          if DEBUG {
              println!("We found no lifecycle configruation for the bucket {}",bucket.to_string());
          }
        }
        returnvar = false;
      }
      else{
        println!("Got some other error asking for the lifecylce {}",e);
        returnvar = false;
      }
    }
  }  
  returnvar
}

/// # get_buckets - gets all buckets and metadata
/// Connects to the UsWest1 Region, not for any particular reason
/// Then requests the list of buckets, which should get buckets from all regions
pub async fn get_buckets(){
    let s3_client = S3Client::new(Region::UsWest1);
    let resp = s3_client.list_buckets().await;
    let resp = resp.unwrap();
    let mut vec = Vec::<BucketMeta>::new();
    for bucket in resp.buckets.unwrap().iter() {
      let bkt:String = bucket.name.clone().unwrap();
      let mut meta_bucket = get_bucket_location(bucket.name.clone().unwrap()).await;
      meta_bucket.contains_lifecycle = has_bucket_lifecycle(&s3_client, &bkt).await;
      meta_bucket.default_encryption = has_encryption_rule(&s3_client, &bkt).await;
      meta_bucket.web_bucket = is_web_bucket(&s3_client,&bkt).await;
      meta_bucket.objects_checked = false;
      vec.push( meta_bucket );
       
    }
   
    for bucket_meta in vec.iter(){
         unsafe{
          if VERBOSE{
            println!("{}", bucket_meta);
          }
        }
          BUCKET_LIST.lock().unwrap().insert(bucket_meta.bucket_name.clone() ,bucket_meta.clone()); 
        
        
        list_items_in_bucket(&s3_client, bucket_meta.bucket_name.as_str() ).await;
    }
    serialize_bucket_meta();
   
  }

  pub async fn has_encryption_rule( s3_client:&S3Client ,bucket:&String)->bool{
    let encryption_result = s3_client.get_bucket_encryption( GetBucketEncryptionRequest{ bucket: bucket.to_string() } ).await;
      match encryption_result{
        Ok(e)=>{ println!("{:#?}",e); 
          //println!("Rules {:?}",e.rules );
          true
        },
        Err(e)=>{
          if  e.to_string().contains("<Code>ServerSideEncryptionConfigurationNotFoundError</Code>")  { 
            false
          }else{
            println!("{:#?}",e); 
            false
          }
        },
        _=>{ 
          println!("Unknown apply encryption rule issue getting encryption config");
          false
        },
      }
  }

  pub async fn is_web_bucket( s3_client:&S3Client ,bucket:&String)->bool{
    let encryption_result = s3_client.get_bucket_website( GetBucketWebsiteRequest{ bucket: bucket.to_string() } ).await;
      match encryption_result{
        Ok(e)=>{ println!("{:#?}",e); 
          //println!("Rules {:?}",e.rules );
          true
        },
        Err(e)=>{
          if  e.to_string().contains("<Code>NoSuchWebsiteConfiguration</Code>")  { 
            false
          }else{
            println!("{:#?}",e); 
            false
          }
        },
        _=>{ 
          println!("Unknown issue getting website configuration");
          false
        },
      }
  }


  pub async fn apply_encryption_rule( s3_client:&S3Client ,bucket:&String, rule:&String){
      if has_encryption_rule( s3_client, bucket ).await {
        unsafe{
          if DEBUG {
            println!("{} already has an encryption rule",bucket);
          }
        }
      }else{
        let sse_rules_vector = vec![ServerSideEncryptionRule{  apply_server_side_encryption_by_default:Some(ServerSideEncryptionByDefault{ sse_algorithm:"AES256".to_string(),kms_master_key_id: None })}];
        let pber = PutBucketEncryptionRequest{ bucket:bucket.to_string(), server_side_encryption_configuration:ServerSideEncryptionConfiguration{rules:sse_rules_vector} };
        let sse_default_result = s3_client.put_bucket_encryption(pber).await;
        match sse_default_result {
          Ok(r)=>{
            println!("bucket {} has had default encryption applied\n{:#?}",bucket,r);
            has_encryption_rule(s3_client, bucket).await;
          },
          Err(e)=>{ println!("bucket {} has an error\n{:#?}",bucket,e)},
          _=>{ println!("Something unexpected happened");},
        }
      }

  }
  
  
  pub fn print_buckets(){
   
    for (_name,bucket_meta) in BUCKET_LIST.lock().unwrap().iter(){
      println!("{}", bucket_meta);
    } 
  }

  pub async fn list_items_in_bucket(client: &S3Client, bucket: &str) {
    let mut list_request = ListObjectsRequest {
        delimiter: Some("/".to_owned()),
        bucket: bucket.to_owned(),
        max_keys: Some(1000),
        ..Default::default()
    };

    // Add error handling here, seems to crash in specific
    let mut response = client
          .list_objects(list_request.clone())
          .await
          .expect("list objects failed");
      
    loop{
      unsafe{
        if DEBUG {
          println!("Args: bucket {}",bucket.to_owned());
        }
      }
      let contents1 = response.contents;
      match contents1{
        Some(_)=> {
          for obj in contents1.unwrap().iter(){
            unsafe{
              if VERBOSE {
                println!("{}", obj.key.as_ref().unwrap());
              }
            }
            //copy_object(client, bucket, obj.key.as_ref().unwrap()).await;
            let head_check_encryption = HeadObjectRequest{ bucket:bucket.to_owned(), key: obj.key.clone().unwrap().to_string().to_owned(),..Default::default() };
            let head_result =  client.head_object(head_check_encryption).await.expect("Failed to retrieve head for object");
            unsafe{
              if DEBUG {  
                println!("{:#?}",head_result);
              }
            }
            match head_result.ssekms_key_id{
              Some(key_id)=>{
                unsafe{
                  if VERBOSE {
                    println!("Encrypted by KMS key {} {}",key_id,obj.key.clone().unwrap().as_str() );
                  }
                }
              },
              _=>{
                 match head_result.server_side_encryption{
                   Some(algo)=>{
                    unsafe{
                      if VERBOSE {
                        println!("Encrypted by SSE-C using {}",algo);
                      }
                    }
                   }
                   _=>{
                    unsafe{
                      if VERBOSE {
                        println!("Not encrypted");
                      }
                    }
                    
                   }
                 }
              }
            }
            
          }
        },
        _ =>{
          unsafe{
            if VERBOSE {
              println!("No objects found");
            }
          }
          
        }
      }
      match response.next_marker {
          Some(_)=>{

          },
          _=>{
            unsafe{
              if DEBUG {
                println!("No further pages of objects");
              }
            }
           
            break;
          }
      }
      list_request.marker = Some(response.next_marker.unwrap());
      list_request.max_keys = Some(1000);
      response = client
          .list_objects(list_request.clone())
          .await
          .expect("list objects failed");
    }
  }

  async fn copy_object(client: &S3Client, bucket: &str, filename: &str) {
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
        .await
        .expect("Couldn't copy object");
    println!("{:#?}", result);
  }
#[cfg(test)]
mod tests {

  extern crate uuid;
  extern crate handlebars;
  extern crate serde_json;
  extern crate tokio;
  extern crate tokio_util;

  use crate::gather::tests::rusoto_s3::DeleteObjectRequest;
use tokio_util::codec::{BytesCodec, FramedRead};
  use rusoto_s3::PutObjectRequest;
  use std::error::Error;
  use rusoto_s3::DeleteBucketRequest;
  use rusoto_s3::CreateBucketRequest;
  use uuid::Uuid;
  use super::*;
  use std::sync::Mutex;
  use tokio::time::{delay_for, Duration};
  use handlebars::Handlebars;
  use std::fs::File;
  use std::io::prelude::*;
  #[macro_use]
  use serde_json::json;
  use log::*;



  lazy_static! {

    static ref S3_CLIENT:S3Client = S3Client::new(Region::UsEast1);
    static ref LOCK:Mutex<bool> =  Mutex::new(true);
  }

  #[actix_rt::test]
  async fn test_copy(){
      let bn = setup_bucket().await;
      create_objects(&bn,Some(1) ).await;
      assert_eq!(true,true);
      teardown_bucket(&bn).await;
  }

  #[actix_rt::test]
  #[ignore]
  async fn test_volume_copy(){
    let bn = setup_bucket().await;
    create_objects(&bn,Some(1001) ).await;
    assert_eq!(true,true);
    teardown_bucket(&bn).await;
  }

  #[actix_rt::test]
  async fn test_transit_policy_exists(){
    let bn = setup_bucket().await;
      assert_eq!(true,true);
      teardown_bucket(&bn).await;
  }

  #[actix_rt::test]
  async fn test_transit_policy_application(){
    let bn = setup_bucket().await;
    let result = transit_policy_template(&bn).unwrap();
    print!("{}",result);
    assert_eq!(true,true);
    teardown_bucket(&bn).await;
  }

  #[actix_rt::test]
  async fn test_encryption_policy_exists(){
    let bn = setup_bucket().await;
    let sse = sse_policy_template().unwrap();
    apply_encryption_rule( &S3_CLIENT ,&bn, &sse).await;
    println!("Default encryption rule to be applied\n{}",sse );
    assert_eq!(true,true);
    teardown_bucket(&bn).await;
  }


  #[actix_rt::test]
  async fn test_encryption_policy_application(){
      let bn = setup_bucket().await;
      assert_eq!(true,true);
      teardown_bucket(&bn).await;
  }


  #[actix_rt::test]
  async fn test_lifecycle_policy_exists(){
      let bn = setup_bucket().await;
      assert_eq!(true,true);
      teardown_bucket(&bn).await;
  }

  #[actix_rt::test]
  async fn test_lifecycle_policy_application(){
    let bn = setup_bucket().await;
    assert_eq!(true,true);
    teardown_bucket(&bn).await;
  }
  
  /// This functions purpose is to create a bunch of objects for a bucket for testing purposes
  async fn create_objects(bn:&String,n:Option<i32>){
    let mut file = File::open("test/fine.jpg").unwrap();
    let mut buf:Vec<u8> = vec![];
    file.read_to_end(&mut buf);
    let x = n.unwrap_or(1);
    loop{
      let ref mut muty = LOCK.try_lock();
      match muty{
        Ok(_) => {
          for y in 0..x {
            let fname = format!("test{}.jpg",y);
            save(fname.as_str(),bn,&S3_CLIENT,buf.clone() ).await;
          }
          break;
        },
        _=>{
          println!("try_lock failed for {} setup",bn);
          delay_for(Duration::from_millis(1000)).await;
        },   
      };
    }
      
    list_items_in_bucket(&S3_CLIENT, bn);
  }

  

  async fn setup_bucket()->String{
    let my_uuid = Uuid::new_v4();
    let bucket_name:String;
    bucket_name = format!("{}{}", "ihtest-",my_uuid);
    let mut create_bucket_result;
    loop{
      let ref mut muty = LOCK.try_lock();
      match muty{
        Ok(_) => {
              create_bucket_result = S3_CLIENT.create_bucket(CreateBucketRequest{bucket:bucket_name.clone(),..Default::default()}).await;
              match create_bucket_result {
                Ok(result)=>{ 
                              println!("{:#?}",result); 
                              break;
                },
                Err(result)=>{  
                              println!("{:#?}",result)  
                            },
              }
              
            },
            _=>{
              println!("try_lock failed for {} setup",bucket_name);
              delay_for(Duration::from_millis(1000)).await;
            },
           
      };
          
    }
    return bucket_name.clone();    
  }
    
    
  async fn add_policy_to_bucket(){
  
  }
   
  async fn save(
      name: &str,
      bucket: &str,
      s3_client:&S3Client,
      buf: Vec<u8>,
  ){
      let put = PutObjectRequest {
          bucket: bucket.to_owned(),
          key: format!("{}", name),
          body: Some(buf.into()),
          ..Default::default()
      };
      let name = name.to_owned();
      let bucket = bucket.to_owned();
      let res = s3_client.put_object(put).await;
      match res{
        Ok(r)=>{
            info!(
              "uploaded {} to {} with version_id: {}",
              name,
              bucket,
              r.version_id.as_deref().unwrap_or_else(|| "-"),
          );
          println!("{:#?}",r);
        },
        Err(r)=>{
          println!("{:#?}",r);
        },
      };
          
  }

  async fn teardown_bucket(bucket_name: &String){
      let delete_bucket_result;
      loop{
        let ref mut muty = LOCK.try_lock();
        match muty{
          Ok(_) => {
              delete_items_in_bucket(&S3_CLIENT, bucket_name).await;
              delete_bucket_result = S3_CLIENT.delete_bucket(DeleteBucketRequest{bucket:bucket_name.to_string() }).await;
              match delete_bucket_result {
                  Ok(result)=>{ println!("{:#?}",result)},
                  Err(result)=>{  println!("{:#?}",result)  }
              }
              break;  
              },
              _=>{
                println!("try_lock failed for {} teardown",bucket_name);
                delay_for(Duration::from_millis(1)).await;
              },
             
        };
            
      }  
    
 
  }


  pub async fn delete_items_in_bucket(client: &S3Client, bucket: &str) {
    let mut list_request = ListObjectsRequest {
        delimiter: Some("/".to_owned()),
        bucket: bucket.to_owned(),
        max_keys: Some(1000),
        ..Default::default()
    };

    // Add error handling here, seems to crash in specific
    let mut response = client
          .list_objects(list_request.clone())
          .await
          .expect("list objects failed");
      
    loop{
      unsafe{
        if DEBUG {
          println!("Args: bucket {}",bucket.to_owned());
        }
      }
      let contents1 = response.contents;
      match contents1{
        Some(_)=> {
          for obj in contents1.unwrap().iter(){
            println!("{}", obj.key.as_ref().unwrap());
            //copy_object(client, bucket, obj.key.as_ref().unwrap()).await;
            let delete_result = client.delete_object( DeleteObjectRequest{ bucket:bucket.to_owned(), key: obj.key.clone().unwrap().to_string().to_owned(),..Default::default() }).await.expect("Failed to retrieve head for object");
            println!("{:#?}",delete_result);
              
            
            
          }
        },
        _ =>{
          
              println!("No objects found to delete");
            
          
          
        }
      }
      match response.next_marker {
          Some(_)=>{

          },
          _=>{
            println!("No further pages of objects");
            break;
          }
      }
      list_request.marker = Some(response.next_marker.unwrap());
      list_request.max_keys = Some(1000);
      response = client
          .list_objects(list_request.clone())
          .await
          .expect("list objects failed");
    }
  }
 
  

}