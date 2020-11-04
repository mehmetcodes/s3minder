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
                               .long("csv")
                               .value_name("FILE")
                               .help("Sets a custom csv file of buckets to check")
                               .takes_value(true))
       .get_matches(); 
  let config = matches.value_of("config").unwrap_or("");
  
  
  match config {
    ""=> { gather::get_buckets().await;},
    _=>{ println!("Will attempt to use {} as a CSV list of buckets",config); }
  }
}



