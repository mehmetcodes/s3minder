#[macro_use] extern crate serde_derive;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_credential;
extern crate rusoto_dynamodb;
extern crate clap;

//use std::default::Default;
use tokio;
mod gather;
mod remediate;
use clap::{Arg, App, SubCommand};
use gather::*;
use remediate::*;
use std::default::Default;
use rusoto_s3::{S3, S3Client };
use rusoto_core::{Region};

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
       .arg(Arg::with_name("lifecycle")
                               .short("l")
                               .long("lifecycle")
                               .value_name("FILE")
                               .help("Uses a custom json file to set the bucket policy")
                               .takes_value(true))
       .arg(Arg::with_name("encryption")
                               .short("e")
                               .long("encryption")
                               .value_name("FILE")
                               .help("Sets a custom json file of the bucket encryption")
                               .takes_value(true))
        .arg(Arg::with_name("transit")
                               .short("t")
                               .long("transit")
                               .value_name("FILE")
                               .help("Sets a custom json file of the bucket to apply transit encryption")
                               .takes_value(true))
        .arg(Arg::with_name("repair")
                                .short("r")
                                .long("repair")
                                .help("Set default encryption policies on all buckets")
                                .takes_value(false))
        .arg(Arg::with_name("v")
                                .short("v")
                                .long("verbose")
                                .help("Set verbose output")
                                .takes_value(false))
        .arg(Arg::with_name("d")
                                .short("d")
                                .long("debug")
                                .help("Set debug output")
                                .takes_value(false))                        
        .get_matches(); 

  let config = matches.value_of("config").unwrap_or("");
  let repair = matches.occurrences_of("repair");
  let debug  = matches.occurrences_of("d");
  let verbose  = matches.occurrences_of("v");

  if repair == 0 {
    println!("This is a dry run which will update you on bucket state.\nIf you would like to apply policy, use the --repair / -r option\n\n");
  }

  unsafe{

    if debug > 0 {
      gather::DEBUG = true;
      println!("debug flag activated")
    }

    if verbose > 0 {
      gather::VERBOSE = true;
      println!("verbose flag activated")
    }
  
  }
  let remediation_options:S3RemediateOptions = Default::default();
  match config {
    ""=> { 
      if debug > 0 {
        println!("{}",repair);
        
      }
        let s3_client = S3Client::new(Region::UsWest1);
        gather::get_buckets(&s3_client).await;
        if repair > 0 {
          remediate::remediate_buckets( &s3_client, remediation_options  ).await;
        }
      },
    _=>{ 
      println!("{}",repair);
      println!("Will attempt to use {} as a CSV list of buckets",config); 
    }
  }
  //print_buckets();
  
  
}



