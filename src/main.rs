extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_credential;

//use std::default::Default;
use std::vec::Vec;
use rusoto_s3::{GetBucketLifecycleRequest,GetBucketLocationRequest};
use rusoto_core::{Region};
use rusoto_s3::{ S3, S3Client};
use tokio;
mod gather;



#[tokio::main]
async fn main() {
  gather::get_buckets().await;
}



