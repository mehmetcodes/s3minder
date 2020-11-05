extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_credential;
extern crate rusoto_dynamodb;
extern crate clap;

//use std::default::Default;
use tokio;
mod gather;
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
    println!("This is a dry run which will update you on bucket state.\nIf you would like to apply policy, use the --repair / -r option");
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

  match config {
    ""=> { 
      println!("{}",repair);
      gather::get_buckets().await;},
    _=>{ 
      println!("{}",repair);
      println!("Will attempt to use {} as a CSV list of buckets",config); 
    }
  }
}



