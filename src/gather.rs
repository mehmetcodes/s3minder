extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_credential;
use rusoto_s3::{GetBucketLifecycleRequest,GetBucketLocationRequest};
use rusoto_core::{Region};
use rusoto_s3::{ S3, S3Client};
use std::fmt;

 

#[derive(Debug,Clone)]
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
          //println!("ok {:?}", val.location_constraint);
          
         
          let together = format!("{}{}{}", b.as_str(), ".s3-",val.location_constraint.clone().unwrap());
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
      /*
      let endpoint_l = s3_client.get_bucket_location( GetBucketLocationRequest{ bucket: bucket.name.clone().unwrap() } ).await;
      match endpoint_l{
        Ok(val) =>{
          //println!("ok {:?}", val.location_constraint);
          let meta_bucket = BucketMeta { 
            bucket_name: bucket.name.clone().unwrap(), 
            bucket_endpoint: ["",&(bucket.name.clone().unwrap()),".s3-",&(val.location_constraint.clone().unwrap()),".amazonaws.com"].join("").to_owned(), 
            contains_lifecycle: false,
            default_encryption: false,
        };
          vec.push( meta_bucket );
        },
        Err(e) => {
          eprintln!("We got an error{}",e);
        }
      }
      */
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
    }
    let mut vec2 = get_encryption_configuration(&vec).await;
    println!("{}",(vec2.pop()).unwrap() );
  }

  async fn get_encryption_configuration(vector:&Vec<BucketMeta> ) -> Vec<BucketMeta>{
    let new_vec = vector.clone();
    return new_vec
  }

  