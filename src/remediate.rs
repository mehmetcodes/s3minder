use super::gather;
use std::default::Default;

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
            applykmskey: true,
        }
    }
}


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




pub fn remediate_buckets( remedy:S3RemediateOptions ){
    for b in super::gather::BUCKET_LIST.lock().unwrap().values(){
      //case skip web buckets
      println!("Test {}",b);
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


