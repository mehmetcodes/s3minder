#[macro_use] extern crate serde_derive;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_credential;
extern crate rusoto_dynamodb;
extern crate clap;
extern crate chrono;


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
use log::{trace,info, warn,debug,error,Record, Level, Metadata,SetLoggerError, LevelFilter};
use chrono::{ Utc};


struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level() && ( metadata.target().contains("s3minder") ) 
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
          println!("{}:{} -- {} - {}",
          record.level(),
          record.target(),
          Utc::now(),
          record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init(lev:LevelFilter) -> Result<(), SetLoggerError> {
  log::set_logger(&LOGGER)
      .map(|()| log::set_max_level(lev))
}

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
  if verbose > 0 {
    let result = init(LevelFilter::Trace);
    match result {
      Ok(e)=>{ 
        info!("verbose flag activated {}",LevelFilter::Trace);
      }
      Err(_)=>{}
    }
    
  }else if debug > 0 {
    //gather::DEBUG = true;
    init(LevelFilter::Debug).expect("Logging init broken");
    info!("debug flag activated")
  }else{
    init(LevelFilter::Info).expect("Logging init broken");
    info!("verbose {}",verbose);
  }



  if repair == 0 {
    info!("This is a dry run which will update you on bucket state.\nIf you would like to apply policy, use the --repair / -r option");
  }

    
  
  let remediation_options:S3RemediateOptions = Default::default();
  match config {
    ""=> { 
      
        debug!("Flag setting: {}",repair);
        
      
        let s3_client = S3Client::new(Region::UsWest1);
        gather::get_buckets(&s3_client).await;
        if repair > 0 {
          remediate::remediate_buckets( &s3_client, remediation_options  ).await;
        }
      },
    _=>{ 
      info!("Config: {}",repair);
      info!("Config: Will attempt to use {} as a CSV list of buckets",config); 
      gather::buckets_from_csv_only(config.to_string());
    }
  }
  info!("Main: Program complete");
  debug!("Debug is on");
  trace!("Trace is on");
  info!("{}",log::max_level());
  
}



