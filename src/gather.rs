extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_credential;

use rusoto_s3::{GetBucketLifecycleRequest,GetBucketLocationRequest,HeadObjectRequest,CopyObjectRequest,ListObjectsRequest};
use rusoto_core::{Region};
use rusoto_s3::{ S3, S3Client};
use std::fmt;




#[derive(Debug,Clone,Default)]
pub struct BucketMeta {
  bucket_name: String,
  bucket_endpoint: String,
  contains_lifecycle: bool,
  default_encryption: bool,
}



impl fmt::Display for BucketMeta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(bucket: {},endpoint: {},had lifecycle {},default encryption {})", self.bucket_name, self.bucket_endpoint,self.contains_lifecycle, self.default_encryption)
    }
}

async fn get_bucket_location(b:String) -> BucketMeta {
    let s3_client = S3Client::new(Region::UsWest1);   
    let endpoint_l = s3_client.get_bucket_location( GetBucketLocationRequest{ bucket: b.clone() } ).await;
    let mut meta_bucket:BucketMeta = BucketMeta{ bucket_name: b.clone(), bucket_endpoint: "Error".to_string(), contains_lifecycle: false, default_encryption: false};
    
    match endpoint_l{
        Ok(val) =>{
          meta_bucket = BucketMeta { 
            bucket_name: b.clone(),  
            bucket_endpoint: ["",b.as_str(),".s3-",&(val.location_constraint.clone().unwrap()),".amazonaws.com"].join("").to_owned(), 
            contains_lifecycle: false,
            default_encryption: false,
            };
        },
        Err(e) => {
          eprintln!("We got an error{}",e);
        }
        
      }
      return meta_bucket
    }
    


pub async fn get_buckets(){
    let s3_client = S3Client::new(Region::UsWest1);
    let resp = s3_client.list_buckets().await;
    let resp = resp.unwrap();
    let mut vec = Vec::<BucketMeta>::new();
    for bucket in resp.buckets.unwrap().iter() {
      //println!("{:?}", bucket.name );
      let meta_bucket = get_bucket_location(bucket.name.clone().unwrap()).await;
      vec.push( meta_bucket );
      let result = s3_client.get_bucket_lifecycle( GetBucketLifecycleRequest { bucket: bucket.name.clone().unwrap() } ).await; 
      match result{
        Ok(r) => { 
          { 
              println!("Rules {:?}",r.rules );
          }
          let mut update_meta= vec.pop().unwrap();
          update_meta.contains_lifecycle = true;
          vec.push(update_meta);
        },
        Err(e) => { 
          if  e.to_string().contains("<Code>NoSuchLifecycleConfiguration</Code>")  { 
            //println!("We have no lifecycle\n\t {}", e );
            let mut update_meta= vec.pop().unwrap();
            update_meta.contains_lifecycle = false;
            vec.push(update_meta);
          }
          else{
            println!("Got some other error asking for the lifecylce {}",e);
          }
        }
      }  
    }
    for bucket_meta in vec.iter(){
         println!("{}", bucket_meta);
         list_items_in_bucket(&s3_client, bucket_meta.bucket_name.as_str() ).await;
    }
    
    
  }


  async fn list_items_in_bucket(client: &S3Client, bucket: &str) {
    let mut list_request = ListObjectsRequest {
        delimiter: Some("/".to_owned()),
        bucket: bucket.to_owned(),
        max_keys: Some(1000),
        ..Default::default()
    };
    let mut response = client
          .list_objects(list_request.clone())
          .await
          .expect("list objects failed");
      
    loop{
      //println!("Items in bucket, page 1: {:#?}", response1);
      println!("Args: bucket {}",bucket.to_owned());
      let contents1 = response.contents;
      match contents1{
        Some(_)=> {
          for obj in contents1.unwrap().iter(){
            println!("{}", obj.key.as_ref().unwrap());
            //copy_object(client, bucket, obj.key.as_ref().unwrap()).await;
            let head_check_encryption = HeadObjectRequest{ bucket:bucket.to_owned(), key: obj.key.clone().unwrap().to_string().to_owned(),..Default::default() };
            let head_result =  client.head_object(head_check_encryption).await.expect("Failed to retrieve head for object");
            println!("{:#?}",head_result);
            
            match head_result.ssekms_key_id{
              Some(key_id)=>{
                println!("Encrypted by KMS key {} {}",key_id,obj.key.clone().unwrap().as_str() );

              },
              _=>{
                 match head_result.server_side_encryption{
                   Some(algo)=>{
                      println!("Encrypted by SSE-C using {}",algo)
                   }
                   _=>{
                     println!("Not encrypted")
                   }
                 }
              }
            }
            
          }
        },
        _ =>{
          println!("No objects found");
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
  use rusoto_s3::DeleteBucketRequest;
  use rusoto_s3::CreateBucketRequest;
  use uuid::Uuid;
  use super::*;

  static mut bucket:Option<String> = None;

  #[actix_rt::test]
  async fn test_copy(){
      setup_bucket().await;
      assert_eq!(true,true);
      teardown_bucket().await;

  }

   async fn setup_bucket(){
    let my_uuid = Uuid::new_v4();
    let mut bucket_name = String::from("");
    unsafe{
      bucket = Some(format!("{}{}", "ihtest-",my_uuid));
      println!("{}",bucket.as_ref().unwrap_or(&String::from("")));
      bucket_name = bucket.clone().unwrap();
    }
    let s3_client = S3Client::new(Region::UsEast1);
    let create_bucket_result = s3_client.create_bucket(CreateBucketRequest{bucket:bucket_name,..Default::default()}).await;
    match create_bucket_result {
      Ok(result)=>{ println!("{:#?}",result)},
      Err(result)=>{  println!("{:#?}",result)  }
    }
  }

  async fn teardown_bucket(){
    let mut bucket_name = String::from("");
    unsafe{
      bucket_name = bucket.clone().unwrap();
    }
    let s3_client = S3Client::new(Region::UsEast1);
    let delete_bucket_result = s3_client.delete_bucket(DeleteBucketRequest{bucket:bucket_name }).await;
    match delete_bucket_result {
      Ok(result)=>{ println!("{:#?}",result)},
      Err(result)=>{  println!("{:#?}",result)  }
    }
  }
}