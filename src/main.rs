extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_credential;
extern crate rusoto_dynamodb;
extern crate clap;

//use std::default::Default;
use std::vec::Vec;
use std::env;
use rusoto_s3::{GetBucketLifecycleRequest,GetBucketLocationRequest};
use rusoto_core::{Region};
use rusoto_s3::{ S3, S3Client};
use tokio;
mod gather;
use rusoto_dynamodb::{DynamoDb, DynamoDbClient, ListTablesInput};
use clap::{Arg, App, SubCommand};



#[tokio::main]
async fn main() {
  let matches = App::new("S3minder")
       .version("0.1")
       .about("Minds your S3 buckets to ensure encryption!")
       .author("Mehmet Yilmaz")
       .arg(Arg::with_name("config")
                               .short("c")
                               .long("config")
                               .value_name("FILE")
                               .help("Sets a custom config file")
                               .takes_value(true))
       .get_matches(); 
 

  gather::get_buckets().await;
}



